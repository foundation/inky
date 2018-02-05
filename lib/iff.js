/**
 * Return `str` if `cond` is truthy. Otherwise, return an empty string. This is used to make
 * conditional renders in component functions simpler to write.
 * @param cond - Condition to evaluate.
 * @param {String} str - String to return if `cond` is truthy.
 * @returns {String} `str` or an empty string.
 */
module.exports = (cond, str) => cond ? str : '';
