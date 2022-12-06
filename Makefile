.PHONY: test nextest clean update doc book serve-book
test: plugins
	cd utils && $(MAKE) test
nextest: plugins
	cd utils && $(MAKE) nextest
clean:
	cd bins && $(MAKE) clean
	cd utils && $(MAKE) clean
	cd plugins && $(MAKE) clean
	cd book && $(MAKE) clean
update:
	cd bins && $(MAKE) update
	cd utils && $(MAKE) update
	cd plugins && $(MAKE) update
doc:
	cd utils && $(MAKE) doc
book:
	cd book && $(MAKE) build
serve-book:
	cd book && $(MAKE) serve

.PHONY: plugins release release-cross
plugins:
	cd plugins && $(MAKE) plugins
release:
	cd bins && $(MAKE) release
release-cross:
	cd bins && $(MAKE) release-cross TARGET=$(TARGET)

EXAMPLES:=Basic Fibonacci Fibonacci2 Gacha Live2D Orga Styles

define example-tpl
.PHONY: example-$(1) example-$(1)-gui example-$(1)-release example-$(1)-gui-release examples/$(1)/config.tex
example-$(1): examples/$(1)/config.yaml plugins
	cd bins && $$(MAKE) run FILE=$$(realpath $$<)
example-$(1)-gui: examples/$(1)/config.yaml plugins
	cd bins && $$(MAKE) run-gui FILE=$$(realpath $$<)
example-$(1)-release: examples/$(1)/config.yaml plugins release
	bins/target/release/ayaka-check $$< --auto
example-$(1)-gui-release: examples/$(1)/config.yaml plugins release
	bins/target/release/ayaka-gui $$<
examples/$(1)/config.tex: examples/$(1)/config.yaml plugins
	cd bins && $$(MAKE) run-latex FILE=$$(realpath $$<)

endef

$(eval $(foreach ex,$(EXAMPLES),$(call example-tpl,$(ex))))

%.pdf: %.tex
	cd $(dir $<) && latexmk -lualatex $(notdir $<)

.SECONDARY:
