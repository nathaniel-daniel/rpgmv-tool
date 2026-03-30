function _0xb1e2(_0x150ba3, _0x5abbfc) {
    const _0x550d7d = _0x550d();
    return _0xb1e2 = function (_0xb1e21e, _0x9b6768) {
        _0xb1e21e = _0xb1e21e - 0xe1;
        let _0x39b0dc = _0x550d7d[_0xb1e21e];
        return _0x39b0dc;
    },
    _0xb1e2(_0x150ba3, _0x5abbfc);
}
function _0x550d() {
    const _0x195681 = ['length', 'match', 'map', 'push', '16536CAQUdq', '47310gvTHQQ', '1320988jqqyMm', '105OvgVwf', 'transformEncryptionKey', '330NNHkLK', '1126863KSjFbm', '10125262aYNRzp', '13423824jMJRAJ', 'hash', '7458384LPuQYT', '40TskQhE'];
    _0x550d = function () {
        return _0x195681;
    };
    return _0x550d();
}
const _0x299e9b = _0xb1e2;
(function (_0x28cd0e, _0x4cf0fc) {
    const _0x5809f1 = _0xb1e2,
    _0x1c4381 = _0x28cd0e();
    while (!![]) {
        try {
            const _0x38ae96 = -parseInt(_0x5809f1(0xea)) / 0x1 + -parseInt(_0x5809f1(0xe9)) / 0x2 * (parseInt(_0x5809f1(0xeb)) / 0x3) + -parseInt(_0x5809f1(0xe8)) / 0x4 * (parseInt(_0x5809f1(0xe3)) / 0x5) + -parseInt(_0x5809f1(0xe2)) / 0x6 + -parseInt(_0x5809f1(0xef)) / 0x7 + parseInt(_0x5809f1(0xf0)) / 0x8 + -parseInt(_0x5809f1(0xee)) / 0x9 * (-parseInt(_0x5809f1(0xed)) / 0xa);
            if (_0x38ae96 === _0x4cf0fc)
                break;
            else
                _0x1c4381['push'](_0x1c4381['shift']());
        } catch (_0x1bb4fc) {
            _0x1c4381['push'](_0x1c4381['shift']());
        }
    }
}
    (_0x550d, 0xe5136), Utils[_0x299e9b(0xec)] = function (_0x5b2111) {
    const _0xbbdfaf = _0x299e9b;
    _0x5b2111 = SparkMD5[_0xbbdfaf(0xe1)](_0x5b2111),
    _0x5b2111 = _0x5b2111[_0xbbdfaf(0xe5)](/.{2}/g);
    for (let _0x1b37a0 = _0x5b2111[_0xbbdfaf(0xe4)] - 0x1; _0x1b37a0 >= 0x0; _0x1b37a0--) {
        _0x5b2111[_0xbbdfaf(0xe7)](_0x5b2111[_0x1b37a0]);
    }
    return _0x5b2111 = _0x5b2111[_0xbbdfaf(0xe6)](_0x359cf7 => parseInt(_0x359cf7, 0x10)),
    _0x5b2111;
});

function transformEncryptionKey(input) {
    let hash = SparkMD5.hash(input);

    let bytes = hash.match(/.{2}/g);

    for (let i = bytes.length - 1; i >= 0; i--) {
        bytes.push(bytes[i]);
    }

    return bytes.map(x => parseInt(x, 16));
}