bosing: 微波波形生成器
=======================

.. toctree::
    :maxdepth: 2
    :caption: 目录:

    quickstart
    instruction
    schedule
    api

简介
----

.. currentmodule:: bosing

``bosing`` 是一个用于生成微波波形的 Python 库, 通过类似 HTML DOM 的结构编排波形,
既可直接用于生成简单的波形序列, 也可作为 `qiskit <https://qiskit.org>`_ 等量子门
线路优化器的后端, 生成复杂的微波波形. 目前实现的功能有:

* 脉冲编排: 通过 :class:`Stack` 等控制波形时序
* 自定义波形: 通过 :class:`Interp` 自定义插值波形

安装
----

.. code-block:: console

    $ pip install bosing


从源码安装
----------

#. 安装 Rust toolchain 1.74+
#. 安装 Python 3.8+
#. Clone 本项目并安装
    .. code-block:: console

        $ git clone https://github.com/kahojyun/Bosing.git
        $ cd Bosing
        $ pip install .

注意
----

.. caution::

    项目中所有相位 (phase) 的单位均为周期数, 比如 :math:`1` 代表弧度制的
    :math:`2\pi`, :math:`0.5` 代表 :math:`\pi`.

示例
----

.. literalinclude:: ../example/schedule.py
    :language: python3
    :linenos:

索引
=====

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`
