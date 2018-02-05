/* eslint-env mocha */

'use strict';

const compare = require('../lib/compare');

describe('Container', () => {
  it('works when parsing a full HTML document', () => {
    const input = `
      <!doctype html>
      <html>
        <head></head>
        <body>
          <Container></Container>
        </body>
      </html>
    `;
    const expected = `
      <!doctype html>
      <html>
        <head></head>
        <body>
          <table align="center" class="container">
            <tbody>
              <tr>
                <td></td>
              </tr>
            </tbody>
          </table>
        </body>
      </html>
    `;
    compare(input, expected);
  });

  it('creates a container table', () => {
    const input = '<Container></Container>';
    const expected = `
      <table align="center" class="container">
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
    const input = '<Row></Row>';
    const expected = `
      <table class="row">
        <tbody>
          <tr></tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a single column with first and last classes', () => {
    const input = '<Column large="12" small="12">One</Column>';
    const expected = `
      <th class="small-12 large-12 columns first last">
        <table>
          <tbody>
            <tr>
              <th>One</th>
              <th class="expander"></th>
            </tr>
          </tbody>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('creates a single column with first and last classes with no-expander', () => {
    const input = '<Column large="12" small="12" no-expander>One</Column>';
    const expected = `
      <th class="small-12 large-12 columns first last">
        <table>
          <tbody>
            <tr>
              <th>One</th>
            </tr>
          </tbody>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('creates a single column with first and last classes with no-expander="false"', () => {
    const input = '<Column large="12" small="12" no-expander="false">One</Column>';
    const expected = `
      <th class="small-12 large-12 columns first last">
        <table>
          <tbody>
            <tr>
              <th>One</th>
              <th class="expander"></th>
            </tr>
          </tbody>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('creates a single column with first and last classes with no-expander="true"', () => {
    const input = '<Column large="12" small="12" no-expander="true">One</Column>';
    const expected = `
      <th class="small-12 large-12 columns first last">
        <table>
          <tbody>
            <tr>
              <th>One</th>
            </tr>
          </tbody>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('creates two columns, one first, one last', () => {
    const input = `
      <Column large="6" small="12">One</Column>
      <Column large="6" small="12">Two</Column>
    `;
    const expected = `
      <th class="small-12 large-6 columns first">
        <table>
          <tbody>
            <tr>
              <th>One</th>
            </tr>
          </tbody>
        </table>
      </th>
      <th class="small-12 large-6 columns last">
        <table>
          <tbody>
            <tr>
              <th>Two</th>
            </tr>
          </tbody>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('creates 3+ columns, first is first, last is last', () => {
    const input = `
      <Column large="4" small="12">One</Column>
      <Column large="4" small="12">Two</Column>
      <Column large="4" small="12">Three</Column>
    `;
    const expected = `
      <th class="small-12 large-4 columns first">
        <table>
          <tbody>
            <tr>
              <th>One</th>
            </tr>
          </tbody>
        </table>
      </th>
      <th class="small-12 large-4 columns">
        <table>
          <tbody>
            <tr>
              <th>Two</th>
            </tr>
          </tbody>
        </table>
      </th>
      <th class="small-12 large-4 columns last">
        <table>
          <tbody>
            <tr>
              <th>Three</th>
            </tr>
          </tbody>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('transfers classes to the final HTML', () => {
    const input = '<Column class="small-offset-8 hide-for-small">One</Column>';
    const expected = `
      <th class="small-offset-8 hide-for-small small-12 large-12 columns first last">
        <table>
          <tbody>
            <tr>
              <th>One</th>
              <th class="expander"></th>
            </tr>
          </tbody>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  // If it just has small, borrow from small for large
  it('automatically assigns large columns if no large attribute is assigned', () => {
    const input = `
      <Column small="4">One</Column>
      <Column small="8">Two</Column>
    `;
    const expected = `
      <th class="small-4 large-4 columns first">
        <table>
          <tbody>
            <tr>
              <th>One</th>
            </tr>
          </tbody>
        </table>
      </th>
      <th class="small-8 large-8 columns last">
        <table>
          <tbody>
            <tr>
              <th>Two</th>
            </tr>
          </tbody>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('automatically assigns small columns as full width if only large defined', () => {
    const input = `
      <Column large="4">One</Column>
      <Column large="8">Two</Column>
    `;
    const expected = `
      <th class="small-12 large-4 columns first">
        <table>
          <tbody>
            <tr>
              <th>One</th>
            </tr>
          </tbody>
        </table>
      </th>
      <th class="small-12 large-8 columns last">
        <table>
          <tbody>
            <tr>
              <th>Two</th>
            </tr>
          </tbody>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('supports nested grids', () => {
    const input = '<Row><Column><Row></Row></Column></Row>';
    const expected = `
      <table class="row">
        <tbody>
          <tr>
            <th class="small-12 large-12 columns first last">
              <table>
                <tbody>
                  <tr>
                    <th>
                      <table class="row">
                        <tbody>
                          <tr></tr>
                        </tbody>
                      </table>
                    </th>
                  </tr>
                </tbody>
              </table>
            </th>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('transfers attributes to the final HTML', () => {
    const input = '<Row dir="rtl"><Column dir="rtl" valign="middle" align="center">One</Column></Row>';
    const expected = `
      <table dir="rtl" class="row">
        <tbody>
          <tr>
            <th class="small-12 large-12 columns first last" dir="rtl" valign="middle" align="center">
              <table>
                <tbody>
                  <tr>
                    <th>One</th>
                    <th class="expander"></th>
                  </tr>
                </tbody>
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
    const input = '<BlockGrid up="4"></BlockGrid>';
    const expected = `
      <table class="block-grid up-4">
        <tbody>
          <tr></tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('copies classes to the final HTML output', () => {
    const input = '<BlockGrid up="4" class="show-for-large"></BlockGrid>';
    const expected = `
      <table class="block-grid up-4 show-for-large">
        <tbody>
          <tr></tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});
