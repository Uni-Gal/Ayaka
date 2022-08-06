import { convertFileSrc, invoke } from "@tauri-apps/api/tauri"
import { Locale } from 'vue-i18n'

export interface OpenGameStatus {
    t: keyof typeof OpenGameStatusType,
    data?: object,
}

export enum OpenGameStatusType {
    LoadProfile,
    CreateRuntime,
    LoadPlugin,
    LoadSettings,
    LoadGlobalRecords,
    LoadRecords,
    Loaded,
}

export interface Settings {
    lang: Locale,
}

export interface RawContext {
    cur_para: string,
    cur_act: number,
    history: Action[],
    bg?: string,
    bgm?: string,
}

export interface Info {
    title: string,
    author: string,
}

export interface Action {
    line: ActionLine[],
    character?: string,
    para_title?: string,
    switches: Switch[],
    props: {
        bg: string | undefined,
        bgm: string | undefined,
        efm: string | undefined,
        voice: string | undefined,
        video: string | undefined,
    },
}

export interface ActionLine {
    type: keyof typeof ActionLineType,
    data: string
}

export enum ActionLineType {
    Chars,
    Block,
}

export interface Switch {
    text: string,
    enabled: boolean,
}

export function open_game(): Promise<void> {
    return invoke("open_game")
}

export function get_settings(): Promise<Settings | undefined> {
    return invoke("get_settings")
}

export function set_settings(settings: Settings): Promise<void> {
    return invoke("set_settings", { settings: settings })
}

export function get_records(): Promise<RawContext[]> {
    return invoke("get_records")
}

export function save_record_to(index: number): Promise<void> {
    return invoke("save_record_to", { index: index })
}

export async function set_locale(loc: Locale): Promise<void> {
    let settings = await get_settings() ?? { lang: "" };
    settings.lang = loc
    await set_settings(settings)
}

export function save_all(): Promise<void> {
    return invoke("save_all")
}

export function choose_locale(locales: Locale[]): Promise<Locale | undefined> {
    return invoke("choose_locale", { locales: locales })
}

export function locale_native_name(loc: Locale): Promise<string> {
    return invoke("locale_native_name", { loc: loc })
}

export function info(): Promise<Info> {
    return invoke("info")
}

export function start_new(locale: Locale): Promise<void> {
    return invoke("start_new", { locale: locale })
}

export function start_record(locale: Locale, index: number): Promise<void> {
    return invoke("start_record", { locale: locale, index: index })
}

export function next_run(): Promise<boolean> {
    return invoke("next_run")
}

export function next_back_run(): Promise<boolean> {
    return invoke("next_back_run")
}

export async function current_run(): Promise<Action | undefined> {
    let res = await invoke<Action | undefined>("current_run")
    if (res) {
        if (res.props.bg) {
            res.props.bg = convertFileSrc(res.props.bg)
        }
        if (res.props.bgm) {
            res.props.bgm = convertFileSrc(res.props.bgm)
        }
        if (res.props.efm) {
            res.props.efm = convertFileSrc(res.props.efm)
        }
        if (res.props.voice) {
            res.props.voice = convertFileSrc(res.props.voice)
        }
        if (res.props.video) {
            res.props.video = convertFileSrc(res.props.video)
        }
    }
    return res
}

export async function current_visited(): Promise<boolean> {
    return invoke("current_visited")
}

export function switch_(i: number): Promise<void> {
    return invoke("switch", { i: i })
}

export function history(): Promise<Action[]> {
    return invoke("history")
}

export function merge_lines(lines: ActionLine[]): string {
    let res = ""
    lines.forEach(s => {
        res += s.data
    })
    return res
}
