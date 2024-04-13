# import matplotlib.pyplot as plt
# import numpy as np
# from scipy.interpolate import make_interp_spline

# import bosing

# # def runge(x):
# #     return 1 / (1 + 25 * x * x)


# def runge(x):
#     return np.cos(np.pi * x)


# x = np.linspace(-0.5, 0.5, 7)
# y = runge(x)
# bs = make_interp_spline(x, y)
# print(bs.t)
# print(bs.c)
# print(bs.k)
# print(y)
# s = bosing.Interp(x, y)
# sx = np.linspace(-0.5, 0.5, 1000)
# sy = np.array([s.sample(v) for v in sx])
# bx = np.linspace(-0.5, 0.5, 1000)
# by = bs(bx)
# rx = np.linspace(-0.5, 0.5, 1000)
# ry = runge(rx)
# plt.plot(sx, sy, label="bosing")
# plt.plot(bx, by, label="scipy")
# plt.plot(rx, ry, label="real")
# plt.legend()
# plt.show()
# max_diff_bosing = np.max(np.abs(sy - ry))
# max_diff_scipy = np.max(np.abs(by - ry))
# print("max_diff_bosing:", max_diff_bosing)
# print("max_diff_scipy:", max_diff_scipy)
# average_diff_bosing = np.mean(np.abs(sy - ry))
# average_diff_scipy = np.mean(np.abs(by - ry))
# print("average_diff_bosing:", average_diff_bosing)
# print("average_diff_scipy:", average_diff_scipy)
