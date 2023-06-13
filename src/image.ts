export function formatImageUrl(url: string, width: number, height: number): string {
    return `image://resize?url=${url}&width=${width}&height=${height}`;
}