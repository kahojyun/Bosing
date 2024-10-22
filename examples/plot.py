import matplotlib.pyplot as plt
import matplotlib.style as mplstyle

from bosing import Stack

mplstyle.use("fast")
plt.figure()
ax = Stack().plot()
ax.autoscale()
plt.show()
