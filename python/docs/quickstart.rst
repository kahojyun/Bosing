快速上手
========

安装
----

.. code-block:: console

    $ pip install bosing


从源码安装
----------

#.  安装 `.NET 8.0 SDK <https://dotnet.microsoft.com/en-us/download>`_
#.  安装 Python 3.8+
#.  Clone 本项目并安装

    .. code-block:: console

        $ git clone https://github.com/kahojyun/Bosing.git
        $ cd Bosing
        $ pip install -e .

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
