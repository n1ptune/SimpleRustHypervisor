WORKDIR := $(shell pwd)

#制作镜像使用的参数
load_addr=0x40200000
entry_point=0x40200000
uboot_path=$(WORKDIR)/bin/u-boot.bin
exe_path=$(WORKDIR)/target/aarch64-unknown-none/debug/SimpleRustHypervisor
hypervisor_path=$(WORKDIR)/bin/SimpleRustHypervisor.elf
binary_path=$(WORKDIR)/bin/SimpleRustHypervisor
hypervisor_img=$(WORKDIR)/bin/SRH_Uimage

.PHONY : build run clean debug
build:
	cargo build 
	cp $(exe_path) $(hypervisor_path)
	llvm-objcopy -O binary -R .note -R .note.gnu.build-id -R .comment -S  $(hypervisor_path) $(binary_path)
	mkimage -A arm64 -O linux -C none -a $(load_addr) -e $(entry_point) -d ${binary_path} ${hypervisor_img}
	
run: build
	qemu-system-aarch64 -cpu cortex-a72 -machine virt,gic-version=3 -smp 1 \
                    -m 512M -nographic \
                    -bios $(uboot_path) \
                    -device loader,file=$(hypervisor_img),addr=0x40200000,force-raw=on \
					-device virtio-serial-device

debug: build
	qemu-system-aarch64 -cpu cortex-a72 -machine virt,gic-version=3 -smp 1 \
                    -m 512M -nographic \
                    -bios $(uboot_path) \
                    -device loader,file=$(hypervisor_img),addr=0x40200000,force-raw=on \
					-device virtio-serial-device \
					-s -S
clean:
	cargo clean
	rm -rf bin/SimpleRustHypervisor
	rm -rf bin/SimpleRustHypervisor.elf
	rm -rf bin/SRH_Uimage