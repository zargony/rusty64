require 'terminal-notifier-guard'

guard :shell do
  watch(/^src\/.*\.rs$/) do |m|
    path = m.first
    mod = path.sub(/^src\//, '')
              .sub(/\/(tests|mod)\.rs$/, '.rs')
              .gsub(/\//, '::')
              .sub(/\.rs$/, '')
    puts "\n\n\n\n=== cargo test #{mod}\n"
    output = `cargo test #{mod}`
    /running \s (\d+) \s tests .+
        test \s result: \s+ (.+) \. \s+
        (\d+) \s+ passed; \s+
        (\d+) \s+ failed; \s+
        (\d+) \s+ ignored;
    /mx.match(output) do |mo|
      tests, result, passed, failed, ignored = mo[1..5]
      if tests.to_i > 0
        n "#{passed} passed, #{failed} failed, #{ignored} ignored", "#{tests} tests: #{result}", failed.to_i > 0 ? :failed : :success
      end
    end
    output
  end
end
