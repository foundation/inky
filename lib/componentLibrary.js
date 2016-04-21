var format = require('util').format;
var $ = require('cheerio');

/**
 * Returns output for desired custom element
 * @param {object} element - Element as a Cheerio object.
 * @returns {string} HTML converted from a custom element to table syntax.
 */
module.exports = {
  // <column>
  columns: function (element) {
    return this.makeColumn(element, 'columns');
  },

  // <row>
  row: function (element) {
    var classes = ['row'];
    if (element.attr('class')) {
      classes = classes.concat(element.attr('class').split(' '));
    }

    return format('<table class="%s"><tbody><tr>%s</tr></tbody></table>', classes.join(' '), element.html());
  },

  // <button>
  button: function (element) {
    var expander = '';
    var inner = element.html();

    // If we have the href attribute we can create an anchor for the inner of the button;
    if (element.attr('href')) {
      inner = format('<a href="%s">%s</a>', element.attr('href'), inner);
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
  },

  // <container>
  container: function (element) {
    var classes = ['container'];
    if (element.attr('class')) {
      classes = classes.concat(element.attr('class').split(' '));
    }

    return format('<table class="%s"><tbody><tr><td>%s</td></tr></tbody></table>', classes.join(' '), element.html());
  },

  // <inky>
  inky: function (element) {
    return '<tr><td><img src="https://raw.githubusercontent.com/arvida/emoji-cheat-sheet.com/master/public/graphics/emojis/octopus.png" /></tr></td>';
  },

  // <block-grid>
  blockGrid: function (element) {
    var classes = ['block-grid', 'up-'+element.attr('up')];
    if (element.attr('class')) {
      classes = classes.concat(element.attr('class').split(' '));
    }
    return format('<table class="%s"><tr>%s</tr></table>', classes.join(' '), element.html());
  },

  // <menu>
  menu: function (element) {
    var classes = ['menu'];
    if (element.attr('class')) {
      classes = classes.concat(element.attr('class').split(' '));
    }
    var centerAttr = element.attr('align') ? 'align="center"' : '';
    return format('<table class="%s"%s><tr><td><table><tr>%s</tr></table></td></tr></table>', classes.join(' '), centerAttr, element.html());
  },

  // <item>
  menuItem: function (element) {
    var classes = ['menu-item'];
    if (element.attr('class')) {
      classes = classes.concat(element.attr('class').split(' '));
    }
    return format('<th class="%s"><a href="%s">%s</a></th>', classes.join(' '), element.attr('href'), element.html());
  },

  // <center>
  center: function (element) {
    if (element.children().length > 0) {
      element.children().each(function() {
        $(this).attr('align', 'center');
        $(this).addClass('float-center');
      });
      element.find('item, .menu-item').addClass('float-center');
    }

    element.attr('data-parsed', '');

    return format('%s', $.html(element));
  },

  // <callout>
  callout: function (element) {
    var classes = ['callout-inner'];
    if (element.attr('class')) {
      classes = classes.concat(element.attr('class').split(' '));
    }

    return format('<table class="callout"><tr><th class="%s">%s</th><th class="expander"></th></tr></table>', classes.join(' '), element.html());
  },

  // <spacer>
  spacer: function (element) {
    var classes = ['spacer'];
    var size = 16;
    if (element.attr('class')) {
      classes = classes.concat(element.attr('class').split(' '));
    }
    if (element.attr('size')) {
      size = (element.attr('size'));
    }

    return format('<table class="%s"><tbody><tr><td height="'+size+'px" style="font-size:'+size+'px;line-height:'+size+'px;">&#xA0;</td></tr></tbody></table>', classes.join(' '), element.html());
  },

  // <wrapper>
  wrapper: function (element) {
    var classes = ['wrapper'];
    if (element.attr('class')) {
      classes = classes.concat(element.attr('class').split(' '));
    }

    return format('<table class="%s" align="center"><tr><td class="wrapper-inner">%s</td></tr></table>', classes.join(' '), element.html());
  }
};