export default {
  extends: ['stylelint-config-standard'],
  rules: {
    'selector-class-pattern': null,
    'custom-property-pattern': null,
    // App scopes coexist in separate windows; their element selectors do not compete.
    'no-descending-specificity': null,
    'color-function-notation': 'modern',
  },
};
