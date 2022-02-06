# RinOS
本项目为《30天自制操作系统》（[日]川合秀实 著）的实现。

## 特点（与原书实现不同之处）

1. 完全采用Rust实现，不再使用原书的nask语言与Rust内联汇编。
2. 使用`bootloader`包实现引导，不再手动实现引导。
3. 使用`x86_64`包实现中断、段表以及CPU端口的读写等，不再使用汇编。
4. 实现64位系统，而不是原书的32位系统。

## 依赖

除了`Cargo.toml`里的依赖外，还需手动安装Rust包**xbuild**、**bootimage**，以及**qemu**。

## 使用

`cd RinOS`

`cargo xrun`

请确保您的Rust环境为**nightly**。

## 参考

https://github.com/yoshitsugu/hariboteos_in_rust \
https://github.com/phil-opp/blog_os
