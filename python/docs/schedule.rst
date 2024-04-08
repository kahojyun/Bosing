波形编排
========

.. currentmodule:: bosing.models

``bosing`` 使用类似 HTML, XAML 的结构来描述波形序列, 用户可以定义基础的波形,
然后通过组合的方式来构建复杂的波形序列. 基础元素有 :doc:`instruction`:

* :class:`Play`
* :class:`ShiftPhase`
* :class:`SetPhase`
* :class:`ShiftFreq`
* :class:`SetFreq`
* :class:`SwapPhase`

另外还有:

* :class:`Repeat`
    根据指定的次数与间隔重复子元素.

* :class:`Barrier`
    用于在 :class:`Stack` 中同步多个通道.

以及布局容器:

* :class:`Stack`
    将多个元素按照顺序排列, 类似于 `Stack Layout
    <https://learn.microsoft.com/en-us/dotnet/maui/user-interface/layouts/stacklayout>`_,
    应该是最常用的容器.

* :class:`Absolute`
    指定子元素相对于容器的绝对时间, 类似于 `Absolute Layout
    <https://learn.microsoft.com/en-us/dotnet/maui/user-interface/layouts/absolutelayout>`_.

* :class:`Grid`
    按照定义的 column 宽度排列子元素, 类似于 `Grid Layout
    <https://learn.microsoft.com/en-us/dotnet/maui/user-interface/layouts/grid>`_.


基础属性
--------

每个元素都有以下基础属性:

:attr:`Element.margin`
    元素前后额外占用的时间, 默认为 ``0``.

:attr:`Element.visibility`
    元素是否生效, 如果为 ``False`` 虽然会占用布局空间, 但是不会执行相应指令.

:attr:`Element.alignment`
    元素在容器中的对齐方式, 目前只有 :class:`Grid` 会使用该属性, 其他布局会忽略
    该属性.

:attr:`Element.duration`
    元素的持续时间, 默认为 ``None``, 由布局系统根据子元素计算.

:attr:`Element.max_duration`
    元素的最大持续时间, 默认为 ``inf``.

:attr:`Element.min_duration`
    元素的最小持续时间, 默认为 ``0``.

.. note::

    布局系统参考的是 XAML 的方案, 相邻元素的 ``margin`` 不会重叠. 例如, 如果两个
    相邻元素的 ``margin`` 分别为 ``0.1`` 和 ``0.2``, 那么它们之间的间隔为
    ``0.3`` 而不是 ``0.2``.

.. note::

    GUI 布局中如果子元素的大小超过了容器的大小, 布局系统会裁剪子元素. 但是在波形
    计算中, 往往需要保留子元素的完整波形, 因此一旦出现超出容器的情况会抛出异常.

.. note:: 

    当 :attr:`Element.duration`, :attr:`Element.max_duration`,
    :attr:`Element.min_duration` 三个属性同时存在且存在冲突时, 优先级为
    ``min_duration > max_duration > duration``.


Stack 布局
----------

在保持子元素前后顺序不变的前提下, 按照 :attr:`Stack.direction` 指定的方向排列子
元素, 默认为 :attr:`ArrangeDirection.BACKWARDS`, 子元素尽量靠后排列. 如果需要同
步多个通道, 可以使用 :class:`Barrier`. 子元素的 :attr:`Element.alignment` 属性会
被忽略, 持续时间尽可能短.


Absolute 布局
-------------

可以指定子元素相对于容器起点的位置. 子元素的 :attr:`Element.alignment` 属性会被
忽略, 持续时间尽可能短. 子元素的位置非负, 容器的持续时间记录为起点到最晚结束的子
元素的终点.

添加子元素时需要指定位置, 默认为 ``0``.

.. code-block:: python

    absolute = Absolute().with_children(
        Play(...),
        (1e-9, Play(...)),
        (2e-9, Stack(...)),
    )


Grid 布局
---------

将容器内部划分为若干列, 每个子元素占据单列或多列, 允许多个子元素占据同一列. 计算
完固定长度与 ``Auto`` 长度的列后, 剩余的长度会按比例分配给 ``Star`` 长度的列. 子
元素在列内部按照 :attr:`Element.alignment` 属性指定的方式排列. 可以利用该布局实
现居中对齐的效果, 还可以配合 :attr:`Play.flexible` 实现根据子元素的持续时间自动
调整背景波形持续时间的效果.

添加子元素时需要指定列, 默认为 ``0``, 以及跨列数, 默认为 ``1``, 如果超出了容器的
列数会当作最后一列处理.

.. code-block:: python

    grid = Grid(columns=["*", "*", "*"]).with_children(
        Play(...),
        (1, Play(...)),
        (0, 3, Stack(...)),
    )

.. tip::

    指定 column 宽度时, 可以使用 ``"Auto"``, ``"*"``, ``"2*"`` 等形式.

    .. code-block:: python

        grid = Grid(columns=["Auto", "*", "2*", 30e-9])

.. caution:: 

    当可用时长较小时, 无法保证按比例分配 ``Star`` 长度的列.


布局算法
--------

布局过程分为两步:

1. Measure 阶段: 给定可用空间, 计算每个元素的最小持续时间. 最顶层的容器的可用空
   间为 ``inf``, 子元素的可用空间由父元素决定.
2. Arrange 阶段: 根据 Measure 阶段计算出的持续时间, 计算每个元素相对于父元素的位
   置. 如果提供的可用空间不足, 会抛出异常.

使用时可以给最顶层的容器指定 :attr:`Element.duration`, 限制布局的持续时间.


执行顺序
--------

.. caution:: 

    在布局完成后, 会按照子元素插入顺序遍历执行, 与布局得到的元素位置无关.


波形对齐
--------

程序在采样包络时会对波形进行对齐, 即将波形的起始点对齐到某个单位时间的整数倍. 对
齐参数在 :attr:`Channel.align_level` 中指定, 假设给定值为 :math:`n`, 采样间隔为
:math:`\Delta t`, 则对齐的单位时间为 :math:`2^n\Delta t`, 比如 ``align_level``
为 -4, 则单位时间为 :math:`2^{-4}\Delta t`, 即 :math:`\Delta t / 16`.

.. note::

    波形对齐目前主要是为了便于重用缓存波形.
