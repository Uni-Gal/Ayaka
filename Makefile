.PHONY: test clean update
test: plugins
	cd utils && $(MAKE) test
clean:
	cd bins && $(MAKE) clean
	cd utils && $(MAKE) clean
	cd plugins && $(MAKE) clean
update:
	cd bins && $(MAKE) update
	cd utils && $(MAKE) update
	cd plugins && $(MAKE) update

.PHONY: plugins release release-cross
plugins:
	cd plugins && $(MAKE) plugins
release:
	cd bins && $(MAKE) release
release-cross:
	cd bins && $(MAKE) release-cross TARGET=$(TARGET)

EXAMPLES:=Fibonacci Fibonacci2 Gacha Markdown Orga

define example-tpl
.PHONY: example-$(1) example-$(1)-gui example-$(1)-release example-$(1)-gui-release examples/$(1)/config.tex
example-$(1): examples/$(1)/config.yaml plugins
	cd bins && $$(MAKE) run FILE=$$(realpath $$<)
example-$(1)-gui: examples/$(1)/config.yaml plugins
	cd bins && $$(MAKE) run-gui FILE=$$(realpath $$<)
example-$(1)-release: examples/$(1)/config.yaml plugins release
	bins/target/release/gal $$< --auto
example-$(1)-gui-release: examples/$(1)/config.yaml plugins release
	bins/target/release/gal-gui $$<
examples/$(1)/config.tex: examples/$(1)/config.yaml
	cd bins && $$(MAKE) run-latex FILE=$$(realpath $$<)

endef

$(eval $(foreach ex,$(EXAMPLES),$(call example-tpl,$(ex))))

%.pdf: %.tex
	cd $(dir $<) && latexmk -lualatex $(notdir $<)

.SECONDARY:
