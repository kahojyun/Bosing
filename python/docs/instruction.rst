波形指令
========

.. currentmodule:: bosing

``bosing`` 通过一系列的指令控制波形的生成, 目前支持的指令有:

:class:`Play`
    在指定通道上添加波形

:class:`ShiftPhase`
    偏置指定通道的相位

:class:`SetPhase`
    设置指定通道的相位

:class:`ShiftFreq`
    偏置指定通道的频率

:class:`SetFreq`
    设置指定通道的频率

:class:`SwapPhase`
    交换两个通道的相位



相位计算
--------

.. note::

    公式中的相位单位均为周期数, 即 :math:`2\pi` 的倍数.

对于每个通道，``bosing`` 记录了以下信息:

- 通道的载波频率 :math:`f_0`
- 由于频率偏置而产生的额外频率 :math:`\Delta f`
- 通道的初相位 :math:`\phi_c`

而 :class:`Play` 指令中还可以指定额外频率 :math:`f_p` 与额外相位 :math:`\phi_p`.
假设经过 DRAG 修正后的波形包络为 :math:`E_d(t)`, 波形起始时间为 :math:`t_0`, 则
混频后的波形为

.. math::

    P(t) = E_d(t) \exp \big[ i 2 \pi ((f_0 + \Delta f) t + f_p (t-t_0) + \phi_c + \phi_p) \big]

涉及相位的指令还有

- :class:`ShiftPhase`:
    改变 :math:`\phi_c`, 与时间无关

- :class:`SetPhase`:
    改变 :math:`\phi_c`, 使得在指定时间 :math:`t` 时相位为 :math:`\phi`. 计算相位时
    只包括 :math:`\Delta f`

- :class:`SwapPhase`:
    改变两个通道的 :math:`\phi_c`, 使得在指定时间 :math:`t` 时两个通道的相位交换. 计算相位时
    包括 :math:`f_0` 与 :math:`\Delta f`

- :class:`ShiftFreq` 与 :class:`SetFreq`:
    改变 :math:`\Delta f`, 同时改变 :math:`\phi_c` 使得在指定时间 :math:`t` 时相位保持连续.
    计算相位时只包括 :math:`\Delta f`
