'use strict';

const $ = require('cheerio');

/**
 * Create an Inky component renderer.
 * @param {Map} library - Components to use.
 * @param {Object} opts - Panini options.
 * @returns {RenderFunction} Render function.
 */
module.exports = (library, opts) => {
  /**
   * Convert a chunk of Inky HTML into plain HTML.
   * @callback RenderFunction
   * @param {Object} element - Cheerio object representing element to transform.
   * @returns {String} Modified HTML.
   */
  return element => {
    // Get the tag name and find the corresponding component
    const tagName = element[0].name;
    const Component = library.get(tagName);

    // If no component exists for this tag, return the HTML wrapped in a table row
    if (!Component) {
      return `<tr><td>${$.html(element, opts.cheerio)}</td></tr>`;
    }

    const props = {
      children: () => element.html()
    };
    const attributes = element.attr();
    const restAttrs = [];

    // Set default props. If an element has any of these as attributes, the defaults will be
    // overridden further down
    if (Component.props) {
      Object.keys(Component.props).forEach(key => {
        props[key] = Component.props[key];
      });
    }

    // Find attributes on the input HTML that matches defined component props
    // Other attributes are collected in a string called `props.rest`
    Object.keys(attributes).forEach(key => {
      if (key in props) {
        const value = attributes[key];

        if (value === 'true') {
          props[key] = true;
        } else if (value === 'false') {
          props[key] = false;
        } else {
          props[key] = attributes[key];
        }
      } else {
        restAttrs.push([key, attributes[key]]);
      }
    });

    props.rest = restAttrs.map(attr => `${attr[0]}="${attr[1]}"`).join(' ');

    return Component.render(props, element, opts).trim();
  };
};
