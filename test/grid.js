var compare = require('./lib/compare');

describe('Container', () => {
  it('works when parsing a full HTML document', () => {
    var input = `
      <!doctype html>
      <html>
        <head></head>
        <body>
          <container></container>
        </body>
      </html>
    `;
    var expected = `
      <!doctype html>
      <html>
        <head></head>
        <body>
          <table class="container" role="presentation">
            <tbody>
              <tr>
                <td></td>
              </tr>
            </tbody>
          </table>
        </body>
      </html>
    `
    compare(input, expected);
  });

  it('creates a container table', () => {
    var input = '<container></container>';
    var expected = `
      <table class="container" role="presentation">
        <tbody>
          <tr>
            <td></td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Grid', () => {
  it('creates a row', () => {
    var input = '<row></row>';
    var expected =  `
      <table class="row" role="presentation">
        <tbody>
          <tr></tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a single column with first and last classes', function () {
    var input = '<columns large="12" small="12">One</columns>';
    var expected = `
      <th class="small-12 large-12 columns first last">
        <table role="presentation">
          <tr>
            <th>One</th>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('creates two columns, one first, one last', function () {
    var input = `
      <columns large="6" small="12">One</columns>
      <columns large="6" small="12">Two</columns>
    `;
    var expected = `
      <th class="small-12 large-6 columns first">
        <table role="presentation">
          <tr>
            <th>One</th>
          </tr>
        </table>
      </th>
      <th class="small-12 large-6 columns last">
        <table role="presentation">
          <tr>
            <th>Two</th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('creates 3+ columns, first is first, last is last', () => {
    var input = `
      <columns large="4" small="12">One</columns>
      <columns large="4" small="12">Two</columns>
      <columns large="4" small="12">Three</columns>
    `;
    var expected = `
      <th class="small-12 large-4 columns first">
        <table role="presentation">
          <tr>
            <th>One</th>
          </tr>
        </table>
      </th>
      <th class="small-12 large-4 columns">
        <table role="presentation">
          <tr>
            <th>Two</th>
          </tr>
        </table>
      </th>
      <th class="small-12 large-4 columns last">
        <table role="presentation">
          <tr>
            <th>Three</th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('transfers classes to the final HTML', () => {
    var input = '<columns class="small-offset-8 hide-for-small">One</columns>';
    var expected = `
      <th class="small-offset-8 hide-for-small small-12 large-12 columns first last">
        <table role="presentation">
          <tr>
            <th>One</th>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected)
  });

  //if it just has small, borrow from small for large
  it('automatically assigns large columns if no large attribute is assigned', () => {
    var input = `
      <columns small="4">One</columns>
      <columns small="8">Two</columns>
    `;
    var expected = `
      <th class="small-4 large-4 columns first">
        <table role="presentation">
          <tr>
            <th>One</th>
          </tr>
        </table>
      </th>
      <th class="small-8 large-8 columns last">
        <table role="presentation">
          <tr>
            <th>Two</th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('automatically assigns small columns as full width if only large defined', () => {
    var input = `
      <columns large="4">One</columns>
      <columns large="8">Two</columns>
    `;
    var expected = `
      <th class="small-12 large-4 columns first">
        <table role="presentation">
          <tr>
            <th>One</th>
          </tr>
        </table>
      </th>
      <th class="small-12 large-8 columns last">
        <table role="presentation">
          <tr>
            <th>Two</th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('supports nested grids', () => {
    var input = '<row><columns><row></row></columns></row>'
    var expected = `
      <table class="row" role="presentation">
        <tbody>
          <tr>
            <th class="small-12 large-12 columns first last">
              <table role="presentation">
                <tr>
                  <th>
                    <table class="row" role="presentation">
                      <tbody>
                        <tr></tr>
                      </tbody>
                    </table>
                  </th>
                </tr>
              </table>
            </th>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Block Grid', () => {
  it('returns the correct block grid syntax', () => {
    var input = '<block-grid up="4"></block-grid>';
    var expected = `
      <table class="block-grid up-4" role="presentation">
        <tr></tr>
      </table>
    `;

    compare(input, expected);
  });

  it('copies classes to the final HTML output', () => {
    var input = '<block-grid up="4" class="show-for-large"></block-grid>';
    var expected = `
      <table class="block-grid up-4 show-for-large" role="presentation">
        <tr></tr>
      </table>
    `;

    compare(input, expected);
  });
});
