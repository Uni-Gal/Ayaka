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

.PHONY: plugins debug-cross release release-cross
plugins:
	cd plugins && $(MAKE) plugins
debug-cross:
	cd bins && $(MAKE) debug-cross TARGET=$(TARGET)
release:
	cd bins && $(MAKE) release
release-cross:
	cd bins && $(MAKE) release-cross TARGET=$(TARGET)

examples/plugins.ayapack: plugins
	(cd -P examples && tar -cf $(abspath $@) -- plugins)

EXAMPLES:=Basic Fibonacci Fibonacci2 Gacha Live2D Orga Pressure Styles

define example-tpl
.PHONY: example-$(1) example-$(1)-gui examples/$(1)/config.tex examples/$(1).ayapack
example-$(1): examples/$(1).ayapack examples/plugins.ayapack
	cd bins && $$(MAKE) run FILE='$$(realpath $$^)'
example-$(1)-gui: examples/$(1).ayapack examples/plugins.ayapack
	cd bins && $$(MAKE) run-gui FILE='$$(realpath $$^)'
examples/$(1)/latex/config.tex: examples/$(1).ayapack examples/plugins.ayapack
	mkdir -p $$(@D)
	cd bins && $$(MAKE) run-latex FILE='$$(realpath $$^)' TEXOUT=$$(abspath $$@)
examples/$(1).ayapack:
	(cd -P examples/$(1) && tar -cf $$(abspath $$@) --exclude=plugins -- *)

endef

$(eval $(foreach ex,$(EXAMPLES),$(call example-tpl,$(ex))))

%.pdf: %.tex
	cd $(dir $<) && latexmk -lualatex $(notdir $<)

.SECONDARY:
