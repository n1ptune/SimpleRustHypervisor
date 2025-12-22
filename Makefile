WORKDIR := $(shell pwd)

#制作镜像使用的参数
load_addr=0x40200000
entry_point=0x40200000
uboot_path=$(WORKDIR)/bin/u-boot.bin
elf_path=$(WORKDIR)/target/aarch64-unknown-none-softfloat/debug/SimpleRustHypervisor
binary_path=$(WORKDIR)/bin/SimpleRustHypervisor
hypervisor_img=$(WORKDIR)/bin/SRH_Uimage
guest_elf=bin/Image

.PHONY : build run clean debug guest

bin/guest.o: $(guest_elf)
	ld.lld -m aarch64elf -r -b binary -o $@ $<

build: bin/guest.o 
	cargo build 
	llvm-objcopy -O binary -R .note -R .note.gnu.build-id -R .comment -S  $(elf_path) $(binary_path)
	mkimage -A arm64 -O linux -C none -a $(load_addr) -e $(entry_point) -d ${binary_path} ${hypervisor_img}

run: build
	../qemu/build/qemu-system-aarch64 -cpu cortex-a72 -machine virt,gic-version=3,virtualization=on -smp 1 \
                    -m 2048M -nographic \
                    -bios $(uboot_path) \
                    -device loader,file=$(hypervisor_img),addr=0x40200000,force-raw=on \
					-device virtio-serial-device

debug: build
	../qemu/build/qemu-system-aarch64 -cpu cortex-a72 -machine virt,gic-version=3,virtualization=on -smp 1 \
                    -m 2048M -nographic \
                    -bios $(uboot_path) \
                    -device loader,file=$(hypervisor_img),addr=0x40200000,force-raw=on \
					-device virtio-serial-device \
					-monitor telnet:127.0.0.1:4444,server,nowait \
					-s -S
clean:
	cargo clean
	rm -rf bin/SimpleRustHypervisor
	rm -rf bin/SimpleRustHypervisor.elf
	rm -rf bin/SRH_Uimage
	rm -rf bin/guest.bin
	rm -rf bin/guest.o