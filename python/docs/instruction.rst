Instruction
===========

.. currentmodule:: pulsegen_client.models

PulseGen 通过一系列的指令控制波形的生成，目前支持的指令有：

:class:`Play`
    在指定通道上添加波形

:class:`ShiftPhase`
    偏置指定通道的相位

:class:`SetPhase`
    设置指定通道的相位

:class:`ShiftFrequency`
    偏置指定通道的频率

:class:`SetFrequency`
    设置指定通道的频率

:class:`SwapPhase`
    交换两个通道的相位



相位计算
--------

目前程序会记录三种频率：通道的载波频率 :math:`f_c`，由于频率偏置而产生的额外频率 :math:`f_a`，以及 :class:`Play` 指令中的附加频率 :math:`f_p`。前两者相加得到全局频率 :math:`f`，即 :math:`f = f_c + f_a`。相位则记录两种：通道的初相位 :math:`\phi_0` 与 :class:`Play` 指令中的附加相位 :math:`\phi_p`。利用这些信息可以计算出时刻 :math:`t` 的额外相位 :math:`\phi_a(t)`：

.. math::

    \phi_a(t) = 2\pi f_a t + \phi_0

载波相位 :math:`\phi_c(t)`：

.. math::

    \phi_c(t) = \phi_a(t) + 2\pi f_c t = 2\pi f t + \phi_0

起始时刻为 :math:`\tau` 的波形中的相位 :math:`\phi_p(t)` 则为：

.. math::

    \phi_p(t) = \phi_c(t) + 2\pi f_p (t - \tau)

目前 :class:`ShiftFrequency` 指令与 :class:`SetFrequency` 指令改变的是 :math:`f_a`，并且会令额外相位 :math:`\phi_a(t)` 在给定时刻 :math:`\tau` 连续：

.. math::

    \phi_a(\tau) = 2\pi f_a \tau + \phi_0 = \phi_a'(\tau) = 2\pi f_a' \tau +
    \phi_0'

:class:`ShiftPhase` 指令无时间参数，直接改变 :math:`\phi_0`。:class:`SetPhase` 改变 :math:`\phi_0` 使得给定时刻 :math:`\tau` 的额外相位 :math:`\phi_a(\tau)` 变为给定值 :math:`\phi`：

.. math::

    \phi_a'(\tau) = 2\pi f_a \tau + \phi_0' = \phi

:class:`SwapPhase` 指令在给定时刻 :math:`\tau` 交换两个通道的载波相位 :math:`\phi_c^{(1)}(\tau)` 与 :math:`\phi_c^{(2)}(\tau)`：

.. math::

    \phi_c^{(1)'}(\tau) = \phi_c^{(2)}(\tau) \\
    \phi_c^{(2)'}(\tau) = \phi_c^{(1)}(\tau)


