(function() {var implementors = {
"ayaka_bindings_types":[["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.PluginType.html\" title=\"struct ayaka_bindings_types::PluginType\">PluginType</a>",1,["ayaka_bindings_types::plugin::PluginType"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.PluginTypeBuilder.html\" title=\"struct ayaka_bindings_types::PluginTypeBuilder\">PluginTypeBuilder</a>",1,["ayaka_bindings_types::plugin::PluginTypeBuilder"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_bindings_types/enum.FrontendType.html\" title=\"enum ayaka_bindings_types::FrontendType\">FrontendType</a>",1,["ayaka_bindings_types::plugin::FrontendType"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.ActionProcessContext.html\" title=\"struct ayaka_bindings_types::ActionProcessContext\">ActionProcessContext</a>",1,["ayaka_bindings_types::plugin::ActionProcessContext"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.ActionProcessResult.html\" title=\"struct ayaka_bindings_types::ActionProcessResult\">ActionProcessResult</a>",1,["ayaka_bindings_types::plugin::ActionProcessResult"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.TextProcessContext.html\" title=\"struct ayaka_bindings_types::TextProcessContext\">TextProcessContext</a>",1,["ayaka_bindings_types::plugin::TextProcessContext"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.TextProcessResult.html\" title=\"struct ayaka_bindings_types::TextProcessResult\">TextProcessResult</a>",1,["ayaka_bindings_types::plugin::TextProcessResult"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.GameProcessContext.html\" title=\"struct ayaka_bindings_types::GameProcessContext\">GameProcessContext</a>",1,["ayaka_bindings_types::plugin::GameProcessContext"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.GameProcessResult.html\" title=\"struct ayaka_bindings_types::GameProcessResult\">GameProcessResult</a>",1,["ayaka_bindings_types::plugin::GameProcessResult"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.LineProcessContext.html\" title=\"struct ayaka_bindings_types::LineProcessContext\">LineProcessContext</a>",1,["ayaka_bindings_types::plugin::LineProcessContext"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.LineProcessResult.html\" title=\"struct ayaka_bindings_types::LineProcessResult\">LineProcessResult</a>",1,["ayaka_bindings_types::plugin::LineProcessResult"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_bindings_types/enum.ActionSubText.html\" title=\"enum ayaka_bindings_types::ActionSubText\">ActionSubText</a>",1,["ayaka_bindings_types::config::ActionSubText"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.RawContext.html\" title=\"struct ayaka_bindings_types::RawContext\">RawContext</a>",1,["ayaka_bindings_types::config::RawContext"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.ActionText.html\" title=\"struct ayaka_bindings_types::ActionText\">ActionText</a>",1,["ayaka_bindings_types::config::ActionText"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_bindings_types/enum.Action.html\" title=\"enum ayaka_bindings_types::Action\">Action</a>",1,["ayaka_bindings_types::config::Action"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.Switch.html\" title=\"struct ayaka_bindings_types::Switch\">Switch</a>",1,["ayaka_bindings_types::config::Switch"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_bindings_types/enum.FileType.html\" title=\"enum ayaka_bindings_types::FileType\">FileType</a>",1,["ayaka_bindings_types::fs::FileType"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_bindings_types/struct.FileMetadata.html\" title=\"struct ayaka_bindings_types::FileMetadata\">FileMetadata</a>",1,["ayaka_bindings_types::fs::FileMetadata"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_bindings_types/enum.FileSeekFrom.html\" title=\"enum ayaka_bindings_types::FileSeekFrom\">FileSeekFrom</a>",1,["ayaka_bindings_types::fs::FileSeekFrom"]]],
"ayaka_model":[["impl Freeze for <a class=\"struct\" href=\"ayaka_model/struct.Settings.html\" title=\"struct ayaka_model::Settings\">Settings</a>",1,["ayaka_model::settings::Settings"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_model/struct.GlobalRecord.html\" title=\"struct ayaka_model::GlobalRecord\">GlobalRecord</a>",1,["ayaka_model::settings::GlobalRecord"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_model/struct.ActionRecord.html\" title=\"struct ayaka_model::ActionRecord\">ActionRecord</a>",1,["ayaka_model::settings::ActionRecord"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_model/enum.OpenGameStatus.html\" title=\"enum ayaka_model::OpenGameStatus\">OpenGameStatus</a>",1,["ayaka_model::view_model::OpenGameStatus"]],["impl&lt;M&gt; Freeze for <a class=\"struct\" href=\"ayaka_model/struct.GameViewModel.html\" title=\"struct ayaka_model::GameViewModel\">GameViewModel</a>&lt;M&gt;<span class=\"where fmt-newline\">where\n    M: Freeze,</span>",1,["ayaka_model::view_model::GameViewModel"]]],
"ayaka_plugin":[["impl&lt;M&gt; Freeze for <a class=\"struct\" href=\"ayaka_plugin/struct.PluginModule.html\" title=\"struct ayaka_plugin::PluginModule\">PluginModule</a>&lt;M&gt;<span class=\"where fmt-newline\">where\n    M: Freeze,</span>",1,["ayaka_plugin::PluginModule"]]],
"ayaka_plugin_nop":[["impl Freeze for <a class=\"struct\" href=\"ayaka_plugin_nop/struct.NopModule.html\" title=\"struct ayaka_plugin_nop::NopModule\">NopModule</a>",1,["ayaka_plugin_nop::NopModule"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_plugin_nop/struct.NopLinker.html\" title=\"struct ayaka_plugin_nop::NopLinker\">NopLinker</a>",1,["ayaka_plugin_nop::NopLinker"]]],
"ayaka_plugin_wasmer":[["impl Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmer/struct.WasmerModule.html\" title=\"struct ayaka_plugin_wasmer::WasmerModule\">WasmerModule</a>",1,["ayaka_plugin_wasmer::WasmerModule"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmer/struct.WasmerLinker.html\" title=\"struct ayaka_plugin_wasmer::WasmerLinker\">WasmerLinker</a>",1,["ayaka_plugin_wasmer::WasmerLinker"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmer/struct.WasmerFunction.html\" title=\"struct ayaka_plugin_wasmer::WasmerFunction\">WasmerFunction</a>",1,["ayaka_plugin_wasmer::WasmerFunction"]],["impl&lt;'a&gt; Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmer/struct.WasmerLinkerHandle.html\" title=\"struct ayaka_plugin_wasmer::WasmerLinkerHandle\">WasmerLinkerHandle</a>&lt;'a&gt;",1,["ayaka_plugin_wasmer::WasmerLinkerHandle"]]],
"ayaka_plugin_wasmi":[["impl Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmi/struct.WasmiModule.html\" title=\"struct ayaka_plugin_wasmi::WasmiModule\">WasmiModule</a>",1,["ayaka_plugin_wasmi::WasmiModule"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmi/struct.WasmiLinker.html\" title=\"struct ayaka_plugin_wasmi::WasmiLinker\">WasmiLinker</a>",1,["ayaka_plugin_wasmi::WasmiLinker"]],["impl&lt;'a&gt; Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmi/struct.WasmiLinkerHandle.html\" title=\"struct ayaka_plugin_wasmi::WasmiLinkerHandle\">WasmiLinkerHandle</a>&lt;'a&gt;",1,["ayaka_plugin_wasmi::WasmiLinkerHandle"]]],
"ayaka_plugin_wasmtime":[["impl Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmtime/struct.WasmtimeModule.html\" title=\"struct ayaka_plugin_wasmtime::WasmtimeModule\">WasmtimeModule</a>",1,["ayaka_plugin_wasmtime::WasmtimeModule"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmtime/struct.WasmtimeLinker.html\" title=\"struct ayaka_plugin_wasmtime::WasmtimeLinker\">WasmtimeLinker</a>",1,["ayaka_plugin_wasmtime::WasmtimeLinker"]],["impl&lt;'a&gt; Freeze for <a class=\"struct\" href=\"ayaka_plugin_wasmtime/struct.WasmtimeLinkerHandle.html\" title=\"struct ayaka_plugin_wasmtime::WasmtimeLinkerHandle\">WasmtimeLinkerHandle</a>&lt;'a&gt;",1,["ayaka_plugin_wasmtime::WasmtimeLinkerHandle"]]],
"ayaka_primitive":[["impl Freeze for <a class=\"enum\" href=\"ayaka_primitive/enum.Line.html\" title=\"enum ayaka_primitive::Line\">Line</a>",1,["ayaka_primitive::line::Line"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_primitive/enum.RawValue.html\" title=\"enum ayaka_primitive::RawValue\">RawValue</a>",1,["ayaka_primitive::raw_value::RawValue"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_primitive/enum.ValueType.html\" title=\"enum ayaka_primitive::ValueType\">ValueType</a>",1,["ayaka_primitive::raw_value::ValueType"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_primitive/struct.Text.html\" title=\"struct ayaka_primitive::Text\">Text</a>",1,["ayaka_primitive::text::Text"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_primitive/enum.SubText.html\" title=\"enum ayaka_primitive::SubText\">SubText</a>",1,["ayaka_primitive::text::SubText"]]],
"ayaka_runtime":[["impl Freeze for <a class=\"struct\" href=\"ayaka_runtime/struct.Paragraph.html\" title=\"struct ayaka_runtime::Paragraph\">Paragraph</a>",1,["ayaka_runtime::config::Paragraph"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_runtime/struct.GameConfig.html\" title=\"struct ayaka_runtime::GameConfig\">GameConfig</a>",1,["ayaka_runtime::config::GameConfig"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_runtime/struct.PluginConfig.html\" title=\"struct ayaka_runtime::PluginConfig\">PluginConfig</a>",1,["ayaka_runtime::config::PluginConfig"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_runtime/struct.Game.html\" title=\"struct ayaka_runtime::Game\">Game</a>",1,["ayaka_runtime::config::Game"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_runtime/struct.Context.html\" title=\"struct ayaka_runtime::Context\">Context</a>",1,["ayaka_runtime::context::Context"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_runtime/enum.OpenStatus.html\" title=\"enum ayaka_runtime::OpenStatus\">OpenStatus</a>",1,["ayaka_runtime::context::OpenStatus"]],["impl&lt;M&gt; Freeze for <a class=\"struct\" href=\"ayaka_runtime/plugin/struct.Module.html\" title=\"struct ayaka_runtime::plugin::Module\">Module</a>&lt;M&gt;<span class=\"where fmt-newline\">where\n    M: Freeze,</span>",1,["ayaka_runtime::plugin::Module"]],["impl&lt;M&gt; Freeze for <a class=\"struct\" href=\"ayaka_runtime/plugin/struct.Runtime.html\" title=\"struct ayaka_runtime::plugin::Runtime\">Runtime</a>&lt;M&gt;",1,["ayaka_runtime::plugin::Runtime"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_runtime/plugin/enum.LoadStatus.html\" title=\"enum ayaka_runtime::plugin::LoadStatus\">LoadStatus</a>",1,["ayaka_runtime::plugin::LoadStatus"]]],
"ayaka_script":[["impl Freeze for <a class=\"struct\" href=\"ayaka_script/struct.ConstParser.html\" title=\"struct ayaka_script::ConstParser\">ConstParser</a>",1,["ayaka_script::grammer::__parse__Const::ConstParser"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_script/struct.ExprParser.html\" title=\"struct ayaka_script::ExprParser\">ExprParser</a>",1,["ayaka_script::grammer::__parse__Expr::ExprParser"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_script/struct.ProgramParser.html\" title=\"struct ayaka_script::ProgramParser\">ProgramParser</a>",1,["ayaka_script::grammer::__parse__Program::ProgramParser"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_script/struct.RefParser.html\" title=\"struct ayaka_script::RefParser\">RefParser</a>",1,["ayaka_script::grammer::__parse__Ref::RefParser"]],["impl Freeze for <a class=\"struct\" href=\"ayaka_script/struct.Program.html\" title=\"struct ayaka_script::Program\">Program</a>",1,["ayaka_script::Program"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_script/enum.Expr.html\" title=\"enum ayaka_script::Expr\">Expr</a>",1,["ayaka_script::Expr"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_script/enum.UnaryOp.html\" title=\"enum ayaka_script::UnaryOp\">UnaryOp</a>",1,["ayaka_script::UnaryOp"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_script/enum.BinaryOp.html\" title=\"enum ayaka_script::BinaryOp\">BinaryOp</a>",1,["ayaka_script::BinaryOp"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_script/enum.ValBinaryOp.html\" title=\"enum ayaka_script::ValBinaryOp\">ValBinaryOp</a>",1,["ayaka_script::ValBinaryOp"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_script/enum.LogicBinaryOp.html\" title=\"enum ayaka_script::LogicBinaryOp\">LogicBinaryOp</a>",1,["ayaka_script::LogicBinaryOp"]],["impl Freeze for <a class=\"enum\" href=\"ayaka_script/enum.Ref.html\" title=\"enum ayaka_script::Ref\">Ref</a>",1,["ayaka_script::Ref"]]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()