export enum MenuId {
    Instances,
    Screenshots,
    Logs,
}

export enum OpenModalType {
    None,
    SideMenu,
    Settings,
}

export enum InstanceType {
    Vanilla,
    Curseforge,
    Modrinth,
    Zip
}

export enum ModloaderType {
    None,
    Fabric,
    Forge
}

export enum Emphasis {
    High,
    Medium,
    Low
}

export function classFromEmphasis(emphasis: Emphasis): string {
    switch (emphasis) {
        case Emphasis.High: return "high-emphasis"
        case Emphasis.Medium: return "medium-emphasis"
        case Emphasis.Low: return "low-emphasis"
    }
}