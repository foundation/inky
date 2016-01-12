Inky
===

Inky is the parser made by ZURB meant to be used with the Foundation for Emails framework. 

It has configurable options:

```
{
  zfTags = {
    button: 'button',
    row: 'row',
    callout: 'callout',
    columns: 'columns',
    subcolumns: 'subcolumns',
    container: 'container',
    inlineListH: 'inline-list-h',
    inlineListV: 'inline-list-v'
  },
  grid = 12
}
```

The zfTags object contains what you'd like to specify as the syntax shortcut. For example, if you wanted to use the word "col" instead of "columns" in your mark up, you can pass it through to Inky like so:

```
var inky = new Inky();
var myConfig = {
  zfTags = {
    button: 'button',
    row: 'row',
    callout: 'callout',
    columns: 'col',
    subcolumns: 'subcolumns',
    container: 'container',
    inlineListH: 'inline-list-h',
    inlineListV: 'inline-list-v'  
  }
}
inky.setConfig(myConfig);
```

Inky relies on Cheerio to load up an HTML string and manipulate it using a series of methods and replacements. If you want to pass initialize Inky, you can start everything off like so:

```
var inky = new Inky();
var syntax = '<column>Here is a column</column>';


var $ = Cheerio.load(syntax);

inky.releaseTheKraken($);
```

You can also add your custom config as an additional argument i.e.
`inky.releaseTheKraken($, myConfig)`.

Repo Contents
=============

* Spec Folder
* index.js


Installation Instructions
=============

1. Clone the repo with `git clone git@github.com:zurb/inky.git`.
2. Navigate into the directory.
3. Run `npm install`. If you don't have node.js installed, downloaded it at [nodejs.org](http://nodejs.org/download/).
4. Run tests with  `jasmine`.

Testing
=============

All the tests need to be started from the root of the project

* Run all tests: `jasmine-node spec`
* Run one test: `jasmine-node spec/testfile`


ZURB
====

Inky was made by [ZURB](http://www.zurb.com), a product design company in Campbell, CA.

If Inky knocks your socks off the way we hope it does and you want more, why not check out [our jobs](http://www.zurb.com/talent)?
