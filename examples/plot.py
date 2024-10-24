import matplotlib.pyplot as plt
import matplotlib.style as mplstyle

from bosing import Barrier, Repeat, Stack

mplstyle.use("fast")
plt.figure()
ax = Stack(Repeat(Barrier("xy", "z", duration=1), 500, 1)).plot()
ax.autoscale()
plt.show()
