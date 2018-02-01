'use strict';

// Grabs all attributes from an element and returns them as a string
// to be put back into outputted table elements
// @returns {string} attributes pulled from inky objects
module.exports = el => {
  const attrs = el.attr();
  const ignoredAttributes = ['class', 'id', 'href', 'size', 'size-sm', 'size-lg', 'large', 'no-expander', 'small', 'target'];
  let result = '';

  for (const key in attrs) {
    if (ignoredAttributes.indexOf(key) === -1) {
      result += ` ${key}="${attrs[key]}"`;
    }
  }

  return result;
};
