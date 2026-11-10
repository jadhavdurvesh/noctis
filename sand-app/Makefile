PYTHON ?= python3
CC ?= gcc
LD ?= ld
SAND_APPS_SDK ?= ../sand-apps
BUILD := build
APP := noctis-engine-probe
APP_VERSION := 0.1.0

.PHONY: all clean inspect
all: $(BUILD)/$(APP).sapp inspect

$(BUILD):
	mkdir -p $(BUILD)

$(BUILD)/start.o: start.S | $(BUILD)
	$(CC) -m32 -c -ffreestanding -fno-pic -fno-pie -fno-stack-protector -o $@ $<

$(BUILD)/$(APP).elf: $(BUILD)/start.o linker.ld
	$(LD) -m elf_i386 -T linker.ld -o $@ $<

$(BUILD)/$(APP).sapp: $(BUILD)/$(APP).elf manifest.json
	test -f "$(SAND_APPS_SDK)/tools/sapp_pack.py"
	$(PYTHON) "$(SAND_APPS_SDK)/tools/sapp_pack.py" $< $@ --name $(APP) --version $(APP_VERSION)

inspect: $(BUILD)/$(APP).sapp
	test -f "$(SAND_APPS_SDK)/tools/sapp_inspect.py"
	$(PYTHON) "$(SAND_APPS_SDK)/tools/sapp_inspect.py" $<

clean:
	rm -rf $(BUILD)
