var format = require('util').format;
var $ = require('cheerio');
var getAttrs = require('./util/getAttrs');

/**
 * Returns output for desired custom element
 * @param {object} element - Element as a Cheerio object.
 * @returns {string} HTML converted from a custom element to table syntax.
 */
module.exports = function(element) {
  var inner = element.html();
  var attrs = getAttrs(element);
  
  switch (element[0].name) {
    // <column>
    case this.components.columns:
      return this.makeColumn(element, 'columns');

    // <row>
    case this.components.row:
      var classes = ['row'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }

      return format('<table %s class="%s"><tbody><tr>%s</tr></tbody></table>&zwj;', attrs, classes.join(' '), inner);

    // <button>
    case this.components.button:
      var expander = '';

      // Prepare optional target attribute for the <a> element
      var target = '';
      if (element.attr('target')) {
        target = ' target=' + element.attr('target');
      }

      // If we have the href attribute we can create an anchor for the inner of the button;
      if (element.attr('href')) {
        inner = format('<a href="%s"%s>%s</a>', element.attr('href'), target, inner);
      }

      // If the button is expanded, it needs a <center> tag around the content
      if (element.hasClass('expand') || element.hasClass('expanded')) {
        inner = format('<center>%s</center>', inner);
        expander = '\n<td class="expander"></td>';
      }

      // The .button class is always there, along with any others on the <button> element
      var classes = ['button'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }

      return format('<table class="%s"><tr><td><table><tr><td>%s</td></tr></table></td>%s</tr></table>', classes.join(' '), inner, expander);

    // <container>
    case this.components.container:
      var classes = ['container'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }

      return format('<table %s class="%s"><tbody><tr><td>%s</td></tr></tbody></table>', attrs, classes.join(' '), inner);

    // <inky>
    case this.components.inky:
      return '<tr><td><img src="https://raw.githubusercontent.com/arvida/emoji-cheat-sheet.com/master/public/graphics/emojis/octopus.png" /></tr></td>';

    // <block-grid>
    case this.components.blockGrid:
      var classes = ['block-grid', 'up-'+element.attr('up')];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }
      return format('<table class="%s"><tr>%s</tr></table>', classes.join(' '), inner);

    // <menu>
    case this.components.menu:
      var classes = ['menu'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }
      var centerAttr = element.attr('align') ? 'align="center"' : '';
      return format('<table %s class="%s"%s><tr><td><table><tr>%s</tr></table></td></tr></table>', attrs, classes.join(' '), centerAttr, inner);

    // <item>
    case this.components.menuItem:
      var classes = ['menu-item'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }
      return format('<th %s class="%s"><a href="%s">%s</a></th>', attrs, classes.join(' '), element.attr('href'), inner);

    // <center>
    case this.components.center:
      if (element.children().length > 0) {
        element.children().each(function() {
          $(this).attr('align', 'center');
          $(this).addClass('float-center');
        });
        element.find('item, .menu-item').addClass('float-center');
      }

      element.attr('data-parsed', '');

      return format('%s', $.html(element, element.options));

    // <callout>
    case this.components.callout:
      var classes = ['callout-inner'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }

      return format('<table %s class="callout"><tr><th class="%s">%s</th><th class="expander"></th></tr></table>', attrs, classes.join(' '), inner);

    // <spacer>
    case this.components.spacer:
      var classes = ['spacer'];
      var size = 16;
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }
      if (element.attr('size')) {
        size = (element.attr('size'));
      }

      return format('<table %s class="%s"><tbody><tr><td height="'+size+'px" style="font-size:'+size+'px;line-height:'+size+'px;">&#xA0;</td></tr></tbody></table>', attrs, classes.join(' '), inner);

    // <wrapper>
    case this.components.wrapper:
      var classes = ['wrapper'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }

      return format('<table %s class="%s" align="center"><tr><td class="wrapper-inner">%s</td></tr></table>', attrs, classes.join(' '), inner);

    default:
      // If it's not a custom component, return it as-is
      return format('<tr><td>%s</td></tr>', $.html(element, element.options));
  }
}
