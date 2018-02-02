'use strict';

module.exports = {
  extract(string) {
    const raws = [];
    let i = 0;
    let raw;
    let str = string;
    const regex = /< *raw *>(.*?)<\/ *raw *>/i;

    while (raw = str.match(regex)) { // eslint-disable-line no-cond-assign
      raws[i] = raw[1];
      str = str.replace(regex, '###RAW' + i + '###');
      i += 1;
    }

    return [raws, str];
  },
  inject(string, raws) {
    let str = string;

    for (const i in raws) {
      if (Object.prototype.hasOwnProperty.call(raws, i)) {
        str = str.replace('###RAW' + i + '###', raws[i]);
      }
    }

    return str;
  }
};
