// Grabs all attributes from an element and returns them as a string
// to be put back into outputted table elements
// @returns {string} attributes pulled from inky objects
module.exports = function(el) {
	var attrs = el.attr();
    var ignoredAttributes = ['class', 'id', 'href', 'size', 'size-sm', 'size-lg', 'large', 'no-expander', 'small', 'target'];
    var result = '';

    for (var key in attrs) {
      if (ignoredAttributes.indexOf(key) == -1) result += (' ' + key + '=' + '"' + attrs[key] + '"');
    }
    return result;
}
