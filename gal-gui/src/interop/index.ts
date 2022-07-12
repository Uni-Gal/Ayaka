import { convertFileSrc, invoke } from "@tauri-apps/api/tauri"
import { Locale } from 'vue-i18n'

export interface OpenGameStatus {
    t: keyof typeof OpenGameStatusType,
    data?: object,
}

export enum OpenGameStatusType {
    LoadSettings,
    LoadRecords,
    LoadProfile,
    CreateRuntime,
    LoadPlugin,
    Loaded,
}

export interface Settings {
    lang: Locale,
}

export interface RawContext {
    cur_para: string,
    cur_act: number,
    history: ActionHistoryData[],
    bg?: string,
    bgm?: string,
}

export interface Info {
    title: string,
    author: string,
}

export interface Action {
    line: string,
    character?: string,
    switches: Switch[],
    bg?: string,
    bgm?: string,
    video?: string,
}

export interface ActionHistoryData {
    line: string,
    character?: string,
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

export async function current_run(): Promise<Action | undefined> {
    let res = await invoke<Action | undefined>("current_run")
    if (res) {
        if (res.bg) {
            res.bg = convertFileSrc(res.bg)
        }
        if (res.bgm) {
            res.bgm = convertFileSrc(res.bgm)
        }
        if (res.video) {
            res.video = convertFileSrc(res.video)
        }
    }
    return res
}

export function switch_(i: number): Promise<void> {
    return invoke("switch", { i: i })
}

export function history(): Promise<ActionHistoryData[]> {
    return invoke("history")
}
