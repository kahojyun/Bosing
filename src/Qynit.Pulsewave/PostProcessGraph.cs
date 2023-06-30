using System.Numerics;

using QuikGraph;
using QuikGraph.Algorithms;

namespace Qynit.Pulsewave;
internal class PostProcessGraph<T>
    where T : unmanaged, INumber<T>, ITrigonometricFunctions<T>
{
    private readonly Dictionary<string, int> _vertexLookup = new();
    private readonly List<ProcessNode> _processNodes = new();
    private readonly AdjacencyGraph<int, Edge<int>> _adjacencyGraph = new();

    internal PulseList<T>.Builder AddSourceNode(string name)
    {
        var node = new SourceNode();
        AddNode(name, node);
        return node.Builder;
    }

    public void AddBasicNode(string name)
    {
        AddNode(name, new ProcessNode());
    }

    public void AddDelay(string name, int delay)
    {
        AddNode(name, new DelayNode(delay));
    }

    public void AddMultiply(string name, IqPair<T> multiplier)
    {
        AddNode(name, new MultiplyNode(multiplier));
    }

    public void AddMatrix(string name, IqPair<T>[,] matrix)
    {
        var inputNames = Enumerable.Range(0, matrix.GetLength(1)).Select(x => $"{name}_in_{x}").ToArray();
        var outputNames = Enumerable.Range(0, matrix.GetLength(0)).Select(x => $"{name}_out_{x}").ToArray();
        var inputIds = inputNames.Select(x => AddNode(x, new ProcessNode())).ToArray();
        var outputIds = outputNames.Select(x => AddNode(x, new ProcessNode())).ToArray();
        var id = AddNode(name, new MatrixNode(matrix, inputIds, outputIds));
        foreach (var inputId in inputIds)
        {
            AddEdge(inputId, id);
        }
        foreach (var outputId in outputIds)
        {
            AddEdge(id, outputId);
        }
    }

    public void AddEdge(string from, string to)
    {
        var fromId = _vertexLookup[from];
        var toId = _vertexLookup[to];
        AddEdge(fromId, toId);
    }

    public PulseList<T> GetPulseList(string name)
    {
        var id = _vertexLookup[name];
        var node = _processNodes[id];
        return node.GetInboxPulseList();
    }

    public void Run()
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
                case MatrixNode matrixNode:
                    RunMatrix(vertex, matrixNode);
                    break;
                default:
                    RunBasic(vertex, node);
                    break;
            }
        }
    }

    private void RunSource(int id, SourceNode node)
    {
        var pulseList = node.Builder.Build();
        node.Inbox.Add((id, pulseList));
        SendPulseListToTargets(id, pulseList);
    }

    private void RunBasic(int id, ProcessNode node)
    {
        SendPulseListToTargets(id, node.GetInboxPulseList());
    }

    private void RunMatrix(int id, MatrixNode node)
    {
        var inputPulseLists = from inputId in node.InputIds
                              join inboxItem in node.Inbox on inputId equals inboxItem.id
                              select inboxItem.pulseList;
        for (var i = 0; i < node.Matrix.GetLength(0); i++)
        {
            var outputPulseList = PulseList<T>.Sum(inputPulseLists.Select((x, j) => x * node.Matrix[i, j]));
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

    private void SendPulseListToTargets(int id, PulseList<T> pulseList)
    {
        foreach (var edge in _adjacencyGraph.OutEdges(id))
        {
            SendPulseListToTarget(id, edge.Target, pulseList);
        }
    }

    private void SendPulseListToTarget(int id, int targetId, PulseList<T> pulseList)
    {
        _processNodes[targetId].Inbox.Add((id, pulseList));
    }

    private void AddEdge(int from, int to)
    {
        _adjacencyGraph.AddEdge(new(from, to));
    }

    private int AddNode(string name, ProcessNode node)
    {
        var id = _processNodes.Count;
        _vertexLookup.Add(name, id);
        _processNodes.Add(node);
        _adjacencyGraph.AddVertex(id);
        return id;
    }

    private record class ProcessNode
    {
        public List<(int id, PulseList<T> pulseList)> Inbox { get; } = new();
        public PulseList<T> GetInboxPulseList()
        {
            return Inbox.Count switch
            {
                0 => PulseList<T>.Empty,
                1 => Inbox[0].pulseList,
                _ => PulseList<T>.Sum(Inbox.Select(x => x.pulseList)),
            };
        }
    }

    private record SourceNode : ProcessNode
    {
        public PulseList<T>.Builder Builder { get; } = new();
    }

    private record DelayNode(int Delay) : ProcessNode;

    private record MultiplyNode(IqPair<T> Multiplier) : ProcessNode;

    private record MatrixNode(IqPair<T>[,] Matrix, int[] InputIds, int[] OutputIds) : ProcessNode;
}
