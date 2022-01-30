# 🌳 balsa

Balsa is a delightfully simple HTML template engine. It is designed to be used in user interfaces such as a CMS where a user needs to be able to edit the template's parameters. Therefore, Balsa includes support for extra metadata, such as friendly variable names, default values, and types.

## What does it look like?

```html
<h1>
  {{ headerText : string, friendlyName: "Header text", defaultValue: "Hello
  world!" }}
</h1>
```

Here, we define a new variable called `headerText`, which is a string with a default value of "Hello world!". We also gave it a friendly name, which can later be resolved and shown to a user in a control panel, etc. We can even define variables inside the template like so:

```html
{{@ defaultHeader : string = "Hello world!" }}
<!-- ... -->
<h1>
  {{ headerText, type: string, friendlyName: "Header text", defaultValue:
  $defaultHeader }}
</h1>
<h2>The default header value is: {{ $defaultHeader }}</h2>
```
