---
raw: true
title: "Style Reference"
---

# Style Reference

Inky includes a built-in SCSS framework for responsive email styles. You can override any variable to customize your emails.

## How to Override

Add a `<style type="text/scss">` block in your layout's `<head>`, or link to an external `.scss` file:

```html
<style type="text/scss">
$primary-color: #ff6600;
$global-width: 640px;
$body-font-family: Georgia, serif;
</style>
```

```html
<link rel="stylesheet" href="theme.scss">
```

Only variables you override are changed — everything else keeps its default value.

## Global

| Variable | Default | Description |
|----------|---------|-------------|
| `$primary-color` | `#2199e8` | Primary brand color (buttons, links, accents) |
| `$secondary-color` | `#777777` | Secondary color |
| `$success-color` | `#3adb76` | Success state color |
| `$warning-color` | `#ffae00` | Warning state color |
| `$alert-color` | `#ec5840` | Alert/error state color |
| `$light-gray` | `#f3f3f3` | Light gray |
| `$medium-gray` | `#cacaca` | Medium gray |
| `$dark-gray` | `#8a8a8a` | Dark gray |
| `$black` | `#0a0a0a` | Black |
| `$white` | `#fefefe` | White |

## Layout

| Variable | Default | Description |
|----------|---------|-------------|
| `$global-width` | `580px` | Maximum email width |
| `$global-width-small` | `95%` | Width on small screens |
| `$global-gutter` | `16px` | Gutter between columns |
| `$global-gutter-small` | `$global-gutter` | Gutter on small screens |
| `$global-padding` | `16px` | Default padding |
| `$global-margin` | `16px` | Default margin |
| `$global-radius` | `3px` | Default border radius |
| `$global-rounded` | `500px` | Fully rounded radius (pill shape) |
| `$global-breakpoint` | `$global-width + $global-gutter` | Responsive breakpoint |
| `$body-background` | `$light-gray` | Page background color |
| `$container-background` | `$white` | Container background color |
| `$container-radius` | `0` | Container border radius |

## Grid

| Variable | Default | Description |
|----------|---------|-------------|
| `$grid-column-count` | `12` | Number of columns in the grid |
| `$column-padding-bottom` | `$global-padding` | Bottom padding on columns |
| `$block-grid-max` | `8` | Maximum items in a block grid |
| `$block-grid-gutter` | `$global-gutter` | Gutter between block grid items |

## Typography

| Variable | Default | Description |
|----------|---------|-------------|
| `$body-font-family` | `Helvetica, Arial, sans-serif` | Body font stack |
| `$global-font-color` | `$black` | Default text color |
| `$global-font-size` | `16px` | Base font size |
| `$global-font-weight` | `normal` | Base font weight |
| `$global-line-height` | `130%` | Base line height |
| `$body-line-height` | `$global-line-height` | Body line height |
| `$header-font-family` | `$body-font-family` | Heading font stack |
| `$header-font-weight` | `$global-font-weight` | Heading font weight |
| `$header-color` | `inherit` | Heading color |
| `$header-margin-bottom` | `10px` | Space below headings |
| `$h1-font-size` | `34px` | H1 size |
| `$h2-font-size` | `30px` | H2 size |
| `$h3-font-size` | `28px` | H3 size |
| `$h4-font-size` | `24px` | H4 size |
| `$h5-font-size` | `20px` | H5 size |
| `$h6-font-size` | `18px` | H6 size |
| `$paragraph-margin-bottom` | `10px` | Space below paragraphs |
| `$lead-font-size` | `20px` | Lead paragraph size |
| `$lead-line-height` | `160%` | Lead paragraph line height |
| `$small-font-size` | `80%` | Small text size |
| `$small-font-color` | `$medium-gray` | Small text color |
| `$text-padding` | `10px` | Text utility padding |
| `$pre-color` | `#ff6908` | Preformatted text color |
| `$font-scale` | `1.2` | Font scale factor |
| `$stat-font-size` | `40px` | Stat/number display size |

### Subheaders

Subheaders are secondary text inside headings, styled with the `<small>` tag:

```html
<h2>Order Confirmed <small>Thank you for your purchase</small></h2>
```

The `<small>` text renders smaller and muted, controlled by these variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `$subheader-lineheight` | `1.4` | Subheader line height |
| `$subheader-color` | `$dark-gray` | Subheader color |
| `$subheader-font-weight` | `$global-font-weight` | Subheader font weight |
| `$subheader-margin-top` | `4px` | Space above subheaders |
| `$subheader-margin-bottom` | `8px` | Space below subheaders |

## Links

| Variable | Default | Description |
|----------|---------|-------------|
| `$anchor-text-decoration` | `none` | Link text decoration |
| `$anchor-color` | `$primary-color` | Link color |
| `$anchor-color-visited` | `$anchor-color` | Visited link color |
| `$anchor-color-hover` | 10% darker than primary | Hover link color |
| `$anchor-color-active` | `$anchor-color-hover` | Active link color |
| `$remove-ios-blue` | `true` | Strip iOS auto-detected link styling (dates, phone numbers, addresses — see [Email Guide](email-guide.md#ios-auto-detected-links)) |

## Buttons

| Variable | Default | Description |
|----------|---------|-------------|
| `$button-background` | `$primary-color` | Button background |
| `$button-color` | `$white` | Button text color |
| `$button-color-alt` | `$medium-gray` | Button alt text color |
| `$button-font-weight` | `bold` | Button font weight |
| `$button-margin` | `0 0 $global-margin 0` | Button margin |
| `$button-border` | `2px solid $button-background` | Button border |
| `$button-radius` | `$global-radius` | Button border radius |
| `$button-rounded` | `$global-rounded` | Pill button radius |

### Button Sizes

The `$button-padding` and `$button-font-size` variables are maps with size keys:

```scss
$button-padding: (
  tiny: 4px 8px,
  small: 5px 10px,
  default: 8px 16px,
  large: 10px 20px
);

$button-font-size: (
  tiny: 10px,
  small: 12px,
  default: 16px,
  large: 20px
);
```

### Button Hover Colors

| Variable | Default | Description |
|----------|---------|-------------|
| `$button-background-hover` | 10% darker than primary | Primary button hover |
| `$button-secondary-background-hover` | 10% lighter than secondary | Secondary button hover |
| `$button-success-background-hover` | 10% darker than success | Success button hover |
| `$button-alert-background-hover` | 10% darker than alert | Alert button hover |
| `$button-warning-background-hover` | 10% darker than warning | Warning button hover |

## Callouts

| Variable | Default | Description |
|----------|---------|-------------|
| `$callout-background` | `$white` | Default callout background |
| `$callout-background-fade` | `85%` | How much to lighten colored callouts |
| `$callout-padding` | `10px` | Callout padding |
| `$callout-padding-small` | `$callout-padding` | Callout padding on small screens |
| `$callout-margin-bottom` | `$global-margin` | Space below callouts |
| `$callout-border` | `1px solid (darkened background)` | Default callout border |
| `$callout-border-primary` | `1px solid (darkened primary)` | Primary callout border |
| `$callout-border-secondary` | `1px solid (darkened secondary)` | Secondary callout border |
| `$callout-border-success` | `1px solid (darkened success)` | Success callout border |
| `$callout-border-warning` | `1px solid (darkened warning)` | Warning callout border |
| `$callout-border-alert` | `1px solid (darkened alert)` | Alert callout border |

## Horizontal Rules / Dividers

| Variable | Default | Description |
|----------|---------|-------------|
| `$hr-width` | `$global-width` | HR width |
| `$hr-border` | `1px solid $black` | HR border style |
| `$hr-margin` | `20px` | HR margin |
| `$hr-align` | `center` | HR alignment |
| `$divider-border` | `1px solid $medium-gray` | Divider border (component) |
| `$divider-margin` | `20px auto` | Divider margin (component) |

## Menu

| Variable | Default | Description |
|----------|---------|-------------|
| `$menu-item-padding` | `10px` | Menu item padding |
| `$menu-item-gutter` | `10px` | Space between menu items |
| `$menu-item-color` | `$primary-color` | Menu item link color |

## Thumbnails

| Variable | Default | Description |
|----------|---------|-------------|
| `$thumbnail-border` | `solid 4px $white` | Thumbnail border |
| `$thumbnail-margin-bottom` | `$global-margin` | Space below thumbnails |
| `$thumbnail-shadow` | `0 0 0 1px rgba($black, 0.2)` | Thumbnail shadow |
| `$thumbnail-shadow-hover` | `0 0 6px 1px rgba($primary-color, 0.5)` | Thumbnail hover shadow |
| `$thumbnail-transition` | `box-shadow 200ms ease-out` | Thumbnail hover transition |
| `$thumbnail-radius` | `$global-radius` | Thumbnail border radius |

## Dark Mode

Dark mode styles are included automatically when you override any of these variables. The build pipeline injects `<meta name="color-scheme">` tags when dark mode styles are detected.

| Variable | Default | Description |
|----------|---------|-------------|
| `$dark-body-background` | `#1a1a1a` | Dark mode page background |
| `$dark-container-background` | `#2d2d2d` | Dark mode container background |
| `$dark-font-color` | `#f0f0f0` | Dark mode text color |
| `$dark-muted-color` | `#a0a0a0` | Dark mode muted text color |
| `$dark-border-color` | `#444444` | Dark mode border color |
| `$dark-link-color` | `#5ab5f7` | Dark mode link color |
