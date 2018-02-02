'use strict';

const $ = require('cheerio');

module.exports = (library, opts) => {
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

    return Component.render(element, props, opts).trim();
  };
};
