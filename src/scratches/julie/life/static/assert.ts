export default function assert(cond: boolean, message?: string): asserts cond {
    if (!cond) {
        console.assert(cond, message || 'assert failed');
        console.trace();  // because on Firefox stack trace is not printed by console.assert()
        throw message || 'assert failed';
    }
}
