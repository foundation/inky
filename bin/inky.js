#!/usr/bin/env node

var chalk     = require('chalk');
var chokidar  = require('chokidar');
var inky      = require('..');
var meow      = require('meow');
var multiline = require('multiline');

var cli = multiline(function() {/*
  Usage
    $ inky <input> <output>

  Options
    -w, --watch   Watch input files for changes
*/});

var aliases = {
  w: 'watch'
}

cli = meow(cli);

if (cli.flags.watch) {
  chokidar.watch(cli.input[0]).on('all', function(evt, file) {
    parse(file);
  });
}
else {
  parse(cli.input[0]);
}

function parse(files) {
  inky({
    src: files,
    dest: cli.input[1]
  }, function() {
    console.log(chalk.magenta(files), 'processed.');
  });
}
