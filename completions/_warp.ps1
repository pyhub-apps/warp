
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'warp' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'warp'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'warp' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('law', 'law', [CompletionResultType]::ParameterValue, 'Search and view laws (국가법령)')
            [CompletionResult]::new('ordinance', 'ordinance', [CompletionResultType]::ParameterValue, 'Search and view local ordinances (자치법규)')
            [CompletionResult]::new('precedent', 'precedent', [CompletionResultType]::ParameterValue, 'Search precedents (판례)')
            [CompletionResult]::new('admrule', 'admrule', [CompletionResultType]::ParameterValue, 'Search administrative rules (행정규칙)')
            [CompletionResult]::new('interpretation', 'interpretation', [CompletionResultType]::ParameterValue, 'Search legal interpretations (법령해석례)')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Unified search across all sources')
            [CompletionResult]::new('config', 'config', [CompletionResultType]::ParameterValue, 'Manage configuration')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Show version information')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;law' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('--size', '--size', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Law type filter')
            [CompletionResult]::new('--law-type', '--law-type', [CompletionResultType]::ParameterName, 'Law type filter')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Department filter')
            [CompletionResult]::new('--department', '--department', [CompletionResultType]::ParameterName, 'Department filter')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for laws')
            [CompletionResult]::new('detail', 'detail', [CompletionResultType]::ParameterValue, 'Get law details')
            [CompletionResult]::new('history', 'history', [CompletionResultType]::ParameterValue, 'Get law history')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;law;search' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('--size', '--size', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;law;detail' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;law;history' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;law;help' {
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for laws')
            [CompletionResult]::new('detail', 'detail', [CompletionResultType]::ParameterValue, 'Get law details')
            [CompletionResult]::new('history', 'history', [CompletionResultType]::ParameterValue, 'Get law history')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;law;help;search' {
            break
        }
        'warp;law;help;detail' {
            break
        }
        'warp;law;help;history' {
            break
        }
        'warp;law;help;help' {
            break
        }
        'warp;ordinance' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('--size', '--size', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'Region filter')
            [CompletionResult]::new('--region', '--region', [CompletionResultType]::ParameterName, 'Region filter')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Law type filter')
            [CompletionResult]::new('--law-type', '--law-type', [CompletionResultType]::ParameterName, 'Law type filter')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for ordinances')
            [CompletionResult]::new('detail', 'detail', [CompletionResultType]::ParameterValue, 'Get ordinance details')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;ordinance;search' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('--size', '--size', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;ordinance;detail' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;ordinance;help' {
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for ordinances')
            [CompletionResult]::new('detail', 'detail', [CompletionResultType]::ParameterValue, 'Get ordinance details')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;ordinance;help;search' {
            break
        }
        'warp;ordinance;help;detail' {
            break
        }
        'warp;ordinance;help;help' {
            break
        }
        'warp;precedent' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('--size', '--size', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Court filter')
            [CompletionResult]::new('--court', '--court', [CompletionResultType]::ParameterName, 'Court filter')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Case type filter')
            [CompletionResult]::new('--case-type', '--case-type', [CompletionResultType]::ParameterName, 'Case type filter')
            [CompletionResult]::new('--date-from', '--date-from', [CompletionResultType]::ParameterName, 'Date from (YYYYMMDD)')
            [CompletionResult]::new('--date-to', '--date-to', [CompletionResultType]::ParameterName, 'Date to (YYYYMMDD)')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for precedents')
            [CompletionResult]::new('detail', 'detail', [CompletionResultType]::ParameterValue, 'Get precedent details')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;precedent;search' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('--size', '--size', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;precedent;detail' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;precedent;help' {
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for precedents')
            [CompletionResult]::new('detail', 'detail', [CompletionResultType]::ParameterValue, 'Get precedent details')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;precedent;help;search' {
            break
        }
        'warp;precedent;help;detail' {
            break
        }
        'warp;precedent;help;help' {
            break
        }
        'warp;admrule' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('--size', '--size', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;interpretation' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('--size', '--size', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;search' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('--size', '--size', [CompletionResultType]::ParameterName, 'Results per page')
            [CompletionResult]::new('-S', '-S ', [CompletionResultType]::ParameterName, 'Source to search (nlic, elis, all)')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Source to search (nlic, elis, all)')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;config' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('set', 'set', [CompletionResultType]::ParameterValue, 'Set a configuration value')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get a configuration value')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Show configuration file path')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize configuration')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;config;set' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;config;get' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;config;path' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;config;init' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;config;help' {
            [CompletionResult]::new('set', 'set', [CompletionResultType]::ParameterValue, 'Set a configuration value')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get a configuration value')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Show configuration file path')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize configuration')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;config;help;set' {
            break
        }
        'warp;config;help;get' {
            break
        }
        'warp;config;help;path' {
            break
        }
        'warp;config;help;init' {
            break
        }
        'warp;config;help;help' {
            break
        }
        'warp;version' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;completions' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'warp;help' {
            [CompletionResult]::new('law', 'law', [CompletionResultType]::ParameterValue, 'Search and view laws (국가법령)')
            [CompletionResult]::new('ordinance', 'ordinance', [CompletionResultType]::ParameterValue, 'Search and view local ordinances (자치법규)')
            [CompletionResult]::new('precedent', 'precedent', [CompletionResultType]::ParameterValue, 'Search precedents (판례)')
            [CompletionResult]::new('admrule', 'admrule', [CompletionResultType]::ParameterValue, 'Search administrative rules (행정규칙)')
            [CompletionResult]::new('interpretation', 'interpretation', [CompletionResultType]::ParameterValue, 'Search legal interpretations (법령해석례)')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Unified search across all sources')
            [CompletionResult]::new('config', 'config', [CompletionResultType]::ParameterValue, 'Manage configuration')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Show version information')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'warp;help;law' {
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for laws')
            [CompletionResult]::new('detail', 'detail', [CompletionResultType]::ParameterValue, 'Get law details')
            [CompletionResult]::new('history', 'history', [CompletionResultType]::ParameterValue, 'Get law history')
            break
        }
        'warp;help;law;search' {
            break
        }
        'warp;help;law;detail' {
            break
        }
        'warp;help;law;history' {
            break
        }
        'warp;help;ordinance' {
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for ordinances')
            [CompletionResult]::new('detail', 'detail', [CompletionResultType]::ParameterValue, 'Get ordinance details')
            break
        }
        'warp;help;ordinance;search' {
            break
        }
        'warp;help;ordinance;detail' {
            break
        }
        'warp;help;precedent' {
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for precedents')
            [CompletionResult]::new('detail', 'detail', [CompletionResultType]::ParameterValue, 'Get precedent details')
            break
        }
        'warp;help;precedent;search' {
            break
        }
        'warp;help;precedent;detail' {
            break
        }
        'warp;help;admrule' {
            break
        }
        'warp;help;interpretation' {
            break
        }
        'warp;help;search' {
            break
        }
        'warp;help;config' {
            [CompletionResult]::new('set', 'set', [CompletionResultType]::ParameterValue, 'Set a configuration value')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get a configuration value')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Show configuration file path')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize configuration')
            break
        }
        'warp;help;config;set' {
            break
        }
        'warp;help;config;get' {
            break
        }
        'warp;help;config;path' {
            break
        }
        'warp;help;config;init' {
            break
        }
        'warp;help;version' {
            break
        }
        'warp;help;completions' {
            break
        }
        'warp;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
