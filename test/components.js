var compare = require('./lib/compare');

describe('Button', function() {
  it('creates a simple button', function() {
    var input = '<button href="http://zurb.com">Button</button>';
    var expected = `
      <table class="button">
        <tr>
          <td>
            <table>
              <tr>
                <td><a href="http://zurb.com">Button</a></td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a button with classes', function() {
    var input = `
      <button class="small alert expand" href="http://zurb.com">Button</button>
    `;
    var expected = `
      <table class="button small alert expand">
        <tr>
          <td>
            <table>
              <tr>
                <td><a href="http://zurb.com">Button</a></td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Menu', function() {
  it('creates a menu with item tags inside', function() {
    var input = `
      <menu>
        <item href="http://zurb.com">Item</item>
      </menu>
    `;
    var expected = `
      <table class="menu">
        <tr>
          <td><a href="http://zurb.com">Item</a></td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('works without using an item tag', function() {
    var input = `
      <menu>
        <td><a href="http://zurb.com">Item 1</a></td>
        <td><a href="http://zurb.com">Item 2</a></td>
      </menu>
    `;
    var expected = `
      <table class="menu">
        <tr>
          <td><a href="http://zurb.com">Item 1</a></td>
          <td><a href="http://zurb.com">Item 2</a></td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});
