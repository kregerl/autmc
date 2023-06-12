const abbreviations = ["k", "M", "B"];

function random(n: number, precision: number): number {
    let prec = 10 ** precision;
    return Math.round(n * prec) / prec;
}

export function formatDownloads(n: number): string {
    let base = Math.floor(Math.log(Math.abs(n)) / Math.log(1000));
    let suffix = abbreviations[Math.min(abbreviations.length - 1, base - 1)];
    base = abbreviations.indexOf(suffix) + 1;
    return suffix ? random(n / 1000 ** base, 2) + suffix : '' + n;
}