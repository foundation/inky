# Inky

[![Build Status](https://travis-ci.org/zurb/inky.svg?branch=master)](https://travis-ci.org/zurb/inky) [![npm version](https://badge.fury.io/js/inky.svg)](https://badge.fury.io/js/inky)

Inky is an HTML-based templating language that converts simple HTML into complex, responsive email-ready HTML. Designed for [Foundation for Emails](http://foundation.zurb.com/emails), a responsive email framework from [ZURB](http://zurb.com).

Give Inky simple HTML like this:

```html
<Row>
  <Column large="6"></Column>
  <Column large="6"></Column>
</Row>
```

And get complicated, but battle-tested, email-ready HTML like this:

```html
<table class="row">
  <tbody>
    <tr>
      <th class="small-12 large-6 columns first">
        <table>
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
      <th class="small-12 large-6 columns first">
        <table>
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
    </tr>
  </tbody>
</table>
```

## Contents

- [Installation](#installation)
- [Usage](#usage)
  - [Standalone](#standalone)
  - [With Gulp](#with-gulp)
  - [Command Line](#command-line)
  - [In the Browser](#in-the-browser)
- [API](#api)
- [Custom Components](#custom-components)
  - [Importing](#importing)
  - [Basics](#basics)
  - [Props](#props)
- [Upgrading from Inky 1.0](#upgrading-from-inky-10)
- [Local Development](#local-development)
- [Related](#related)
- [License](#license)

## Installation

```bash
npm install inky
```

## Usage

Inky can be used standalone, as a Gulp plugin, or with a CLI. You can also access the `Inky` parser class directly.

### Standalone

To transform a bunch of Inky files:

```js
var inky = require('inky');

inky({
  src: 'src/pages/**/*.html',
  dest: 'dist'
}).then(() => {
  console.log('Done parsing.');
});
```

To transform a single string:

```js
const inky;

const parser = new inky.Inky();
const output = parser.releaseTheKraken('<Row>...</Row>');
```

### With Gulp

```js
var inky = require('inky')

function parse() {
  gulp.src('src/pages/**/*.html')
    .pipe(inky())
    .pipe(gulp.dest('dist'));
}
```

### Command Line

Install [inky-cli](https://github.com/zurb/inky-cli) to get the `inky` command. The first option is a glob of input files, and the second option is a folder to output them to. Add the `--watch` flag to re-compile when files are added or changed.

```bash
npm install inky-cli --global
inky src/pages/**/*.html dist --watch
```

Doesn't support advanced settings at the moment.

### In the Browser

If you're using Webpack or another module bundler, import `inky/browser` to get a browser-friendly version of Inky.

```js
const Inky = require('inky/browser');

const parser = new Inky();
const output = parser.releaseTheKraken('<Row>...</Row>');
```

You can also access the parser at `window.Inky`.

## API

### `inky([opts])`

Parse a set of Inky HTML files and output them to a folder. Or, transform the files in a Gulp stream from Inky HTML to plain HTML.

- **opts** (Object): Plugin options.
  - **src** (String): [Glob](https://www.npmjs.com/package/glob) of files to process. You don't need to supply this when using Inky with Gulp.
  - **dest** (String): Folder to output processed files to. You don't need to supply this when using Inky with Gulp.
  - Pass options to the `Inky` constructor here as well.

When used standalone, returns a Promise which resolves when all files have been parsed. When used in a Gulp stream, returns a stream transform function.

### `new inky.Inky([opts])`

Create an Inky parser.

- **opts** (Object) Parser options.
  - **cheerio** (Object): [Cheerio](https://www.npmjs.com/package/cheerio) settings.
  - **columnCount** (Number): Column count for the grid. Make sure your Foundation for Emails project has the same column count in the Sass as well.
  - **components** (Array of Objects): Custom components to use. See [Custom Components](#custom-components).

#### `.releaseTheKraken(input)`

Convert Inky HTML into plain HTML.

- **input** (String): Input HTML. It can be a fragment of HTML or a full document.

Returns converted HTML as a String.

## Custom Components

You can add your own custom components to the Inky parser. A component is a function that takes the attributes on the custom element (referred to here as "props"), and returns a string of new HTML to replace the original code with.

### Importing

There are two ways to bring in custom components. They can be passed to the options object as an array:

```js
const inky = require('inky');

inky({
  src: 'src/*.html',
  dest: 'dest',
  components: [{ /* ...component... */ }, { /* ... */ }]
})
```

Or, you can point to a folder with custom components, one per file. Each component should be a `.js` file with a `module.exports` containing the component definition. Note that this approach doesn't work if you're using Inky in a browser environment, because there's no filesystem.

```js
const inky = require('inky');

inky({
  src: 'src/*.html',
  dest: 'dest',
  components: 'src/components/'
})
```

### Basics

A component looks something like this:

```js
module.exports = {
  name: 'Thing',
  props: {
    class: ''
  },
  render(props) {
    return `
      <table class="${props.class}" ${props.rest}>
        ${props.children()}
      </table>
    `;
  }
}
```

The `name` property is the name of the tag. We like using title case, to make the custom tags stand out from normal HTML. In the above example, the component would be rendered by writing `<Thing></Thing>`.

The `render()` function is where the magic happens. It's a function that returns the HTML your custom component generates. It's called with three parameters, but in most cases you'll only need the first one.

- `props` is an object containing the HTML attributes used on the component.
- `element` is the Cheerio object for this component instance.
- `options` is the Panini parser options.

### Props

Components can also have props. These are the attributes set on the root element of the component. For example, the `<Column>` component built-in to Inky has the `small` and `large` props to set the size of the column. These props are used to calculate the final HTML.

You can define props for your component. Props can be built-in HTML attributes (like `class`) or custom attributes (like `small` and `large`). In the above example, we just define a `class` prop. Note that it's an object: the key is `class` and the value is `''`. That's the default value. If an instance of `<Thing>` doesn't have a `class` attribute, this default value will be used instead.

All of a component's props are collected into the `props` object, which is passed to the `render()` function. If an instance of a component has any other attributes not defined in `props`, they're added to `props.rest`. For example, a user might want to set an `id` on a `<Thing>`, or add one of Mailchimp's custom attributes. Our component needs to copy over these attributes to the final HTML, even though we aren't going to use them directly.

Lastly, all components have a `children` prop. It's a function that returns the HTML inside the component instance. If a component is meant to be a wrapper of some kind, you'll need this.

## Upgrading from Inky 1.0

In terms of HTML output, Inky 2.0 is identical to 1.0. However, there's one major change: all the custom tags are now CamelCase instead of lowercase. This was done to prevent Inky HTML from clashing with regular HTML, and to make the custom elements stand out in your code.

- `<block-grid>` => `<BlockGrid>`
- `<button>` => `<Button>`
- `<callout>` => `<Callout>`
- `<center>` => `<Center>`
- `<columns>` => `<Column>`
- `<container>` => `<Container>`
- `<divider>` => `<Divider>`
- `<item>` => `<Item>`
- `<menu>` => `<Menu>`
- `<row>` => `<Row>`
- `<spacer>` => `<Spacer>`
- `<wrapper>` => `<Wrapper>`

## Local Development

```bash
git clone https://github.com/zurb/inky
cd inky
npm install
```

To test in a Node environment, run `npm run test:node`. To test in a browser environment, run `npm run test:browser`. Testing in a browser enviornment requires Node.js 6 or higher.

## Related

- **[inky-cli](https://github.com/zurb/inky-cli)** - CLI for Inky
- **[Foundation for Emails](http://foundation.zurb.com/inky)** - Responsive HTML email framework

## License

MIT &copy; [ZURB](https://zurb.com)
