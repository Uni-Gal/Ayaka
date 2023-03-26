import { invoke } from "@tauri-apps/api/tauri"
import { Locale } from 'vue-i18n'

export var DIST_PORT: number = -1

export async function init(): Promise<void> {
    DIST_PORT = await invoke("dist_port")
}

export function conv_src(path?: string): string | undefined {
    if (path) {
        return decodeURIComponent(`http://127.0.0.1:${DIST_PORT}/fs${path}`)
    }
    return undefined
}

export interface OpenGameStatus {
    t: keyof typeof OpenGameStatusType,
    data?: object,
}

export enum OpenGameStatusType {
    LoadProfile,
    CreateRuntime,
    LoadPlugin,
    GamePlugin,
    LoadResource,
    LoadParagraph,
    LoadSettings,
    LoadGlobalRecords,
    LoadRecords,
    Loaded,
}

export interface Settings {
    lang: Locale,
    sub_lang?: Locale,
    bgm_volume: number,
    voice_volume: number,
    video_volume: number,
}

export interface RawContext {
    cur_para: string,
    cur_act: number,
    locals: {
        bg?: string,
        bgm?: string,
        ch_models?: string,
    }
}

export interface GameInfo {
    title: string,
    author: string,
    props: {
        bg?: string,
    },
}

export interface Action {
    type: keyof typeof ActionType,
    data: undefined | ActionText | Switch[] | CustomVars
}

export enum ActionType {
    Empty,
    Text,
    Switches,
    Custom,
}

export interface ActionText {
    text: ActionLine[],
    ch_key?: string,
    character?: string,
    vars: {
        voice?: string
    }
}

export interface CustomVars {
    video?: string,
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

export function ayaka_version(): Promise<string> {
    return invoke("ayaka_version")
}

export function open_game(): Promise<void> {
    return invoke("open_game")
}

export function get_settings(): Promise<Settings> {
    return invoke("get_settings")
}

export function set_settings(settings: Settings): Promise<void> {
    return invoke("set_settings", { settings: settings })
}

export function get_records(): Promise<ActionText[]> {
    return invoke("get_records")
}

export function save_record_to(index: number): Promise<void> {
    return invoke("save_record_to", { index: index })
}

export async function set_locale(loc: Locale): Promise<void> {
    let settings = await get_settings()
    settings.lang = loc
    await set_settings(settings)
}

export async function set_sub_locale(loc?: Locale): Promise<void> {
    let settings = await get_settings()
    settings.sub_lang = loc
    await set_settings(settings)
}

export async function set_volumes(bgm: number, voice: number, video: number): Promise<void> {
    let settings = await get_settings()
    settings.bgm_volume = bgm
    settings.voice_volume = voice
    settings.video_volume = video
    await set_settings(settings)
}

export function save_all(): Promise<void> {
    return invoke("save_all")
}

export function avaliable_locale(locales: Locale[]): Promise<Locale[]> {
    return invoke("avaliable_locale", { locales: locales })
}

export function choose_locale(locales: Locale[]): Promise<Locale | undefined> {
    return invoke("choose_locale", { locales: locales })
}

export async function info(): Promise<GameInfo> {
    let res = await invoke<GameInfo | undefined>("info");
    return res ?? { title: "", author: "", props: {} }
}

export function start_new(): Promise<void> {
    return invoke("start_new", {})
}

export function start_record(index: number): Promise<void> {
    return invoke("start_record", { index: index })
}

export function next_run(): Promise<boolean> {
    return invoke("next_run")
}

export function next_back_run(): Promise<boolean> {
    return invoke("next_back_run")
}

export function current_run(): Promise<RawContext | undefined> {
    return invoke("current_run")
}

export function current_action(): Promise<[Action, Action | undefined] | undefined> {
    return invoke("current_action")
}

export function current_title(): Promise<string | undefined> {
    return invoke("current_title")
}

export async function current_visited(): Promise<boolean> {
    return invoke("current_visited")
}

export function switch_(i: number): Promise<void> {
    return invoke("switch", { i: i })
}

export function history(): Promise<[Action, Action | undefined][]> {
    return invoke("history")
}

export function merge_lines(lines: ActionLine[]): string {
    let res = ""
    lines.forEach(s => {
        res += s.data
    })
    return res
}
