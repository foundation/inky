var format = require('util').format;
var $ = require('cheerio');

/**
 * Returns output for desired custom element
 * @param {object} element - Element as a Cheerio object.
 * @returns {string} HTML converted from a custom element to table syntax.
 */
module.exports = function(element) {
  var component = $(element);
  var inner     = $(element).html();

  switch (element.name) {
    // <column>
    case this.components.columns:
      return this.makeColumn(component, 'columns');

    // <row>
    case this.components.row:
      var classes = ['row'];
      if (component.attr('class')) {
        classes = classes.concat(component.attr('class').split(' '));
      }

      return format('<table class="%s"><tbody><tr>%s</tr></tbody></table>', classes.join(' '), inner);

    // <callout>
    case this.components.callout:
      var classes = ['callout'];
      if (component.attr('class')) {
        classes = classes.concat(component.attr('class').split(' '));
      }

      if (component.parent() && this.isTableCell(component.parent()[0].name)) {
        return format('<table><tbody><tr><td class="%s">%s</td></tr></tbody></table>', classes.join(' '), inner);
      }
      else {
        return format('<td class="%s">%s</td>', classes.join(' '), inner);
      }

    // <button>
    case this.components.button:
      // If we have the href attribute we can create an anchor for the inner of the button;
      if (component.attr('href')) {
        inner = format('<a href="%s">%s</a>', component.attr('href'), inner);
      }

      // The .button class is always there, along with any others on the <button> element
      var classes = ['button'];
      if (component.attr('class')) {
        classes = classes.concat(component.attr('class').split(' '));
      }

      return format('<table class="%s"><tr><td><table><tr><td>%s</td></tr></table></td></tr></table>', classes.join(' '), inner);

    // <container>
    case this.components.container:
      var classes = ['container'];
      if (component.attr('class')) {
        classes = classes.concat(component.attr('class').split(' '));
      }

      return format('<table class="%s"><tbody><tr><td>%s</td></tr></tbody></table>', classes.join(' '), inner);

    // <inky>
    case this.components.inky:
      return '<tr><td><img src="https://raw.githubusercontent.com/arvida/emoji-cheat-sheet.com/master/public/graphics/emojis/octopus.png" /></tr></td>';

    // <block-grid>
    case this.components.blockGrid:
      return format('<table class="block-grid up-%s"><tr>%s</tr></table>', component.attr('up'), inner);

    // <menu>
    case this.components.menu:
      return format('<table class="menu"><tr>%s</tr></table>', inner);

    // <item>
    case this.components.menuItem:
      return format('<td><a href="%s">%s</a></td>', component.attr('href'), inner);

    default:
      // If it's not a custom component, return it as-is
      return format('<tr><td>%s</td></tr>', $.html(element));
  }
}
