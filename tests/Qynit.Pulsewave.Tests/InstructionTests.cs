namespace Qynit.Pulsewave.Tests
{
    public class InstructionTests
    {
        [Fact]
        public void Instruction_New()
        {
            var name = "Test";
            var channels = new Channel[] { "ch0" };
            var instruction = new Instruction(name, channels);

            // Assert
            Assert.Equal(name, instruction.Name);
            Assert.Equal(channels, instruction.Channels);
        }
    }
}
