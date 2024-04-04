using System.Numerics;

using QuikGraph;
using QuikGraph.Algorithms;

namespace Bosing;
public class PostProcessTransform(BosingOptions options)
{
    private readonly List<ProcessNode> _processNodes = [];
    private readonly List<int> _terminalIds = [];
    private readonly AdjacencyGraph<int, Edge<int>> _adjacencyGraph = new();
    private readonly BosingOptions _options = options;

    public int AddSourceNode(PulseList pulseList)
    {
        return AddNode(new SourceNode(pulseList) { Options = _options });
    }

    public int AddTerminalNode(out int resultId)
    {
        var id = AddSimpleNode();
        resultId = _terminalIds.Count;
        _terminalIds.Add(id);
        return id;
    }

    public int AddSimpleNode()
    {
        return AddNode(new ProcessNode() { Options = _options });
    }

    public int AddDelay(double delay)
    {
        return AddNode(new DelayNode(delay) { Options = _options });
    }

    public int AddMultiply(Complex multiplier)
    {
        return AddNode(new MultiplyNode(multiplier) { Options = _options });
    }

    public int AddFilter(SignalFilter<double> filter)
    {
        return AddNode(new FilterNode(filter) { Options = _options });
    }

    public void AddMatrix(Complex[,] matrix, out int[] inputIds, out int[] outputIds)
    {
        var inputLength = matrix.GetLength(1);
        var outputLength = matrix.GetLength(0);
        inputIds = Enumerable.Range(0, inputLength).Select(_ => AddSimpleNode()).ToArray();
        outputIds = Enumerable.Range(0, outputLength).Select(_ => AddSimpleNode()).ToArray();
        var id = AddNode(new MatrixNode(matrix, inputIds, outputIds) { Options = _options });
        foreach (var inputId in inputIds)
        {
            AddEdge(inputId, id);
        }
        foreach (var outputId in outputIds)
        {
            AddEdge(id, outputId);
        }
    }

    public bool AddEdge(int from, int to)
    {
        return _adjacencyGraph.AddEdge(new(from, to));
    }

    public List<PulseList> Finish()
    {
        foreach (var vertex in _adjacencyGraph.TopologicalSort())
        {
            var node = _processNodes[vertex];
            switch (node)
            {
                case SourceNode sourceNode:
                    RunSource(vertex, sourceNode);
                    break;
                case DelayNode delayNode:
                    RunDelay(vertex, delayNode);
                    break;
                case MultiplyNode multiplyNode:
                    RunMultiply(vertex, multiplyNode);
                    break;
                case FilterNode filterNode:
                    RunFilter(vertex, filterNode);
                    break;
                case MatrixNode matrixNode:
                    RunMatrix(vertex, matrixNode);
                    break;
                default:
                    RunBasic(vertex, node);
                    break;
            }
        }
        return _terminalIds.Select(x => _processNodes[x].GetInboxPulseList()).ToList();
    }

    private void RunSource(int id, SourceNode node)
    {
        var pulseList = node.PulseList;
        node.Inbox.Add((id, pulseList));
        SendPulseListToTargets(id, pulseList);
    }

    private void RunBasic(int id, ProcessNode node)
    {
        SendPulseListToTargets(id, node.GetInboxPulseList());
    }

    private void RunFilter(int id, FilterNode node)
    {
        var pulseList = node.GetInboxPulseList().Filtered(node.Filter);
        SendPulseListToTargets(id, pulseList);
    }

    private void RunMatrix(int id, MatrixNode node)
    {
        var inputPulseLists = from inputId in node.InputIds
                              join inboxItem in node.Inbox on inputId equals inboxItem.id
                              select inboxItem.pulseList;
        for (var i = 0; i < node.Matrix.GetLength(0); i++)
        {
            var outputPulseList = PulseList.Sum(inputPulseLists.Select((x, j) => x * node.Matrix[i, j]), _options.TimeTolerance, _options.AmpTolerance);
            SendPulseListToTarget(id, node.OutputIds[i], outputPulseList);
        }
    }

    private void RunMultiply(int id, MultiplyNode node)
    {
        var pulseList = node.GetInboxPulseList() * node.Multiplier;
        SendPulseListToTargets(id, pulseList);
    }

    private void RunDelay(int id, DelayNode node)
    {
        var pulseList = node.GetInboxPulseList().TimeShifted(node.Delay);
        SendPulseListToTargets(id, pulseList);
    }

    private void SendPulseListToTargets(int id, PulseList pulseList)
    {
        foreach (var edge in _adjacencyGraph.OutEdges(id))
        {
            SendPulseListToTarget(id, edge.Target, pulseList);
        }
    }

    private void SendPulseListToTarget(int id, int targetId, PulseList pulseList)
    {
        _processNodes[targetId].Inbox.Add((id, pulseList));
    }

    private int AddNode(ProcessNode node)
    {
        var id = _processNodes.Count;
        _processNodes.Add(node);
        _adjacencyGraph.AddVertex(id);
        return id;
    }

    private record class ProcessNode
    {
        public List<(int id, PulseList pulseList)> Inbox { get; } = [];
        public required BosingOptions Options { get; init; }
        public PulseList GetInboxPulseList()
        {
            return Inbox.Count switch
            {
                0 => PulseList.Empty,
                1 => Inbox[0].pulseList,
                _ => PulseList.Sum(Inbox.Select(x => x.pulseList), Options.TimeTolerance, Options.AmpTolerance),
            };
        }
    }

    private record SourceNode(PulseList PulseList) : ProcessNode;

    private record DelayNode(double Delay) : ProcessNode;

    private record MultiplyNode(Complex Multiplier) : ProcessNode;

    private record FilterNode(SignalFilter<double> Filter) : ProcessNode;

    private record MatrixNode(Complex[,] Matrix, int[] InputIds, int[] OutputIds) : ProcessNode;
}
