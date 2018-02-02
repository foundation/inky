/* eslint-env mocha */
/* eslint-disable no-useless-escape */

'use strict';

const compare = require('./lib/compare');

describe('Center', () => {
  it('applies a text-center class and center alignment attribute to the first child', () => {
    const input = `
      <Center>
        <div></div>
      </Center>
    `;
    const expected = `
      <center>
        <div align="center" class="float-center"></div>
      </center>
    `;

    compare(input, expected);
  });

  it(`doesn't choke if center tags are nested`, () => {
    const input = `
      <Center>
        <Center>
        </Center>
      </Center>
    `;

    const expected = `
      <center>
        <center align="center" class="float-center">
        </center>
      </center>
    `;

    compare(input, expected);
  });

  it('applies the class float-center to <Item> elements', () => {
    const input = `
      <Center>
        <Menu>
          <Item href="#"></Item>
        </Menu>
      </Center>
    `;

    const expected = `
      <center>
        <table align="center" class="menu float-center">
          <tbody>
            <tr>
              <td>
                <table>
                  <tbody>
                    <tr>
                      <th class="menu-item float-center"><a href="#"></a></th>
                    </tr>
                  </tbody>
                </table>
              </td>
            </tr>
          </tbody>
        </table>
      </center>
    `;

    compare(input, expected);
  });
});

describe('Button', () => {
  it('creates a simple button', () => {
    const input = '<Button href="http://zurb.com">Button</Button>';
    const expected = `
      <table class="button">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>
                    <td><a href="http://zurb.com">Button</a></td>
                  </tr>
                </tbody>
              </table>
            </td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a button with target="_blank" attribute', () => {
    const input = '<Button href="http://zurb.com" target="_blank">Button</Button>';
    const expected = `
      <table class="button">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>
                    <td><a href="http://zurb.com" target="_blank">Button</a></td>
                  </tr>
                </tbody>
              </table>
            </td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a button with classes', () => {
    const input = `
      <Button class="small alert" href="http://zurb.com">Button</Button>
    `;
    const expected = `
      <table class="button small alert">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>
                    <td><a href="http://zurb.com">Button</a></td>
                  </tr>
                </tbody>
              </table>
            </td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a correct expanded button', () => {
    const input = `
      <Button class="expand" href="http://zurb.com">Button</Button>
    `;
    const expected = `
      <table class="button expand">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>
                    <td>
                      <center><a href="http://zurb.com" align="center" class="float-center">Button</a></center>
                    </td>
                  </tr>
                </tbody>
              </table>
            </td>
            <td class="expander"></td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Menu', () => {
  it('creates a menu with item tags inside', () => {
    const input = `
      <Menu>
        <Item href="http://zurb.com">Item</Item>
      </Menu>
    `;
    const expected = `
      <table class="menu">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>
                    <th class="menu-item"><a href="http://zurb.com">Item</a></th>
                  </tr>
                </tbody>
              </table>
            </td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a menu with items tags inside, containing target="_blank" attribute', () => {
    const input = `
      <Menu>
        <Item href="http://zurb.com" target="_blank">Item</Item>
      </Menu>
    `;
    const expected = `
      <table class="menu">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>
                    <th class="menu-item"><a href="http://zurb.com" target="_blank">Item</a></th>
                  </tr>
                </tbody>
              </table>
            </td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a menu with classes', () => {
    const input = `
      <Menu class="vertical">
      </Menu>
    `;
    const expected = `
      <table class="menu vertical">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>
                  </tr>
                </tbody>
              </table>
            </td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('works without using an item tag', () => {
    const input = `
      <Menu>
        <th class="menu-item"><a href="http://zurb.com">Item 1</a></th>
      </Menu>
    `;
    const expected = `
      <table class="menu">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>
                    <th class="menu-item"><a href="http://zurb.com">Item 1</a></th>
                  </tr>
                </tbody>
              </table>
            </td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Callout', () => {
  it('creates a callout with correct syntax', () => {
    const input = '<Callout>Callout</Callout>';
    const expected = `
      <table class="callout">
        <tbody>
          <tr>
            <th class="callout-inner">Callout</th>
            <th class="expander"></th>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('copies classes to the final HTML', () => {
    const input = '<Callout class="primary">Callout</Callout>';
    const expected = `
      <table class="callout">
        <tbody>
          <tr>
            <th class="callout-inner primary">Callout</th>
            <th class="expander"></th>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Spacer', () => {
  it('creates a spacer element with correct size', () => {
    const input = '<spacer size="10"></spacer>';
    const expected = `
      <table class="spacer">
        <tbody>
          <tr>
            <td height="10px" style="font-size:10px;line-height:10px;">&nbsp;</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a spacer with a default size or no size defined', () => {
    const input = '<spacer></spacer>';
    const expected = `
      <table class="spacer">
        <tbody>
          <tr>
            <td height="16px" style="font-size:16px;line-height:16px;">&nbsp;</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a spacer element for small screens with correct size', () => {
    const input = '<spacer size-sm="10"></spacer>';
    const expected = `
      <table class="spacer hide-for-large">
        <tbody>
          <tr>
            <td height="10px" style="font-size:10px;line-height:10px;">&nbsp;</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a spacer element for large screens with correct size', () => {
    const input = '<spacer size-lg="20"></spacer>';
    const expected = `
      <table class="spacer show-for-large">
        <tbody>
          <tr>
            <td height="20px" style="font-size:20px;line-height:20px;">&nbsp;</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a spacer element for small and large screens with correct sizes', () => {
    const input = '<spacer size-sm="10" size-lg="20"></spacer>';
    const expected = `
      <table class="spacer hide-for-large">
        <tbody>
          <tr>
            <td height="10px" style="font-size:10px;line-height:10px;">&nbsp;</td>
          </tr>
        </tbody>
      </table>
      <table class="spacer show-for-large">
        <tbody>
          <tr>
            <td height="20px" style="font-size:20px;line-height:20px;">&nbsp;</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('copies classes to the final spacer HTML', () => {
    const input = '<spacer size="10" class="bgcolor"></spacer>';
    const expected = `
      <table class="spacer bgcolor">
        <tbody>
          <tr>
            <td height="10px" style="font-size:10px;line-height:10px;">&nbsp;</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});

describe('wrapper', () => {
  it('creates a wrapper that you can attach classes to', () => {
    const input = `<wrapper class="header"></wrapper>`;
    const expected = `
      <table class="wrapper header" align="center">
        <tbody>
          <tr>
            <td class="wrapper-inner"></td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});

describe('divider', () => {
  it('creates a horizontal rule that you can attach classes to', () => {
    const input = `<Divider class="dotted">`;
    const expected = `
      <table class="h-line dotted">
        <tr>
          <th>&nbsp;</th>
        </tr>
      </table>
    `;
    compare(input, expected);
  });
});

describe('raw', () => {
  it('creates a wrapper that ignores anything inside', () => {
    const input = `<raw><<LCG Program\TG LCG Coupon Code Default='246996'>></raw>`;
    const expected = `<<LCG Program\TG LCG Coupon Code Default='246996'>>`;

    compare(input, expected);
  });
});
