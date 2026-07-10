export default {
  extends: ['stylelint-config-standard-vue'],
  ignoreFiles: ['dist/**', 'node_modules/**', 'src/assets/fonts/**'],
  rules: {
    'at-rule-no-unknown': [true, { ignoreAtRules: ['theme'] }],
    'custom-property-pattern': null,
    'selector-class-pattern': null,
  },
}
