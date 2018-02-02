'use strict';

module.exports = (library, opts) => {
  return element => {
    // Get the tag name and find the corresponding component
    const tagName = element[0].name;
    const Component = library.get(tagName);

    // If no component exists for this tag, return the HTML as-is
    if (!Component) {
      return false;
    }

    const props = {
      children: element.html(),
      class: Component.class || []
    };
    const attributes = element.attr();
    const restAttrs = [];

    // Find attributes on the input HTML that matches defined component props
    // Other attributes are collected in a string called `props.rest`
    Object.keys(attributes).forEach(key => {
      if (Component.props && key in Component.props) {
        props[key] = attributes[key];
      } else if (key === 'class') {
        props.class.push.apply(props.class, attributes[key].split(' '));
      } else {
        restAttrs.push([key, attributes[key]]);
      }
    });

    props.rest = restAttrs.map(attr => `${attr[0]}="${attr[1]}"`);

    return Component.render(element, props, opts);
  };
};
