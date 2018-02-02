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

## Installation

```bash
npm install inky
```

## Usage

Inky can be used standalone, as a Gulp plugin, or with a CLI. You can also access the `Inky` parser class directly.

### Standalone

```js
var inky = require('inky');

inky({
  src: 'src/pages/**/*.html',
  dest: 'dist'
}).then(() => {
  console.log('Done parsing.');
});
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

## API

### `inky([opts])`

Parse a set of Inky HTML files and output them to a folder. Or, transform the files in a Gulp stream from Inky HTML to plain HTML.

- **opts** (Object) Plugin options.
  - **src** (String): [Glob](https://www.npmjs.com/package/glob) of files to process. You don't need to supply this when using Inky with Gulp.
  - **dest** (String): Folder to output processed files to. You don't need to supply this when using Inky with Gulp.
  - **columnCount** (Number): Column count for the grid. Make sure your Foundation for Emails project has the same column count in the Sass as well.
  - **cheerio** (Object): [Cheerio](https://www.npmjs.com/package/cheerio) settings.

When used standalone, returns a Promise which resolves when all files have been parsed. When used in a Gulp stream, returns a stream transform function.

## Related

- **[inky-cli](https://github.com/zurb/inky-cli)** - CLI for Inky
- **[Foundation for Emails](http://foundation.zurb.com/inky)** - Responsive HTML email framework

## License

&copy; [ZURB](https://zurb.com)
