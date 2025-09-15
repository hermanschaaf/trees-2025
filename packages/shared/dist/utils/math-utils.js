export function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
}
export function lerp(a, b, t) {
    return a + (b - a) * t;
}
export function randomSeed() {
    return Math.floor(Math.random() * 1000000);
}
export function generateId() {
    return Math.random().toString(36).substr(2, 9);
}
export function debounce(func, delay) {
    let timeoutId;
    return (...args) => {
        if (timeoutId)
            clearTimeout(timeoutId);
        timeoutId = setTimeout(() => func(...args), delay);
    };
}
