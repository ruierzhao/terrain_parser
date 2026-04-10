let pos = 368
const bytesPerIndex = 2
if (pos % bytesPerIndex !== 0) {
    pos += bytesPerIndex - (pos % bytesPerIndex);
}
console.log('>> pos:', pos);
