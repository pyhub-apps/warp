# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_warp_global_optspecs
	string join \n v/verbose f/format= h/help V/version
end

function __fish_warp_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_warp_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_warp_using_subcommand
	set -l cmd (__fish_warp_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c warp -n "__fish_warp_needs_command" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_needs_command" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_needs_command" -s V -l version -d 'Print version'
complete -c warp -n "__fish_warp_needs_command" -f -a "law" -d 'Search and view laws (국가법령)'
complete -c warp -n "__fish_warp_needs_command" -f -a "ordinance" -d 'Search and view local ordinances (자치법규)'
complete -c warp -n "__fish_warp_needs_command" -f -a "precedent" -d 'Search precedents (판례)'
complete -c warp -n "__fish_warp_needs_command" -f -a "admrule" -d 'Search administrative rules (행정규칙)'
complete -c warp -n "__fish_warp_needs_command" -f -a "interpretation" -d 'Search legal interpretations (법령해석례)'
complete -c warp -n "__fish_warp_needs_command" -f -a "search" -d 'Unified search across all sources'
complete -c warp -n "__fish_warp_needs_command" -f -a "config" -d 'Manage configuration'
complete -c warp -n "__fish_warp_needs_command" -f -a "version" -d 'Show version information'
complete -c warp -n "__fish_warp_needs_command" -f -a "completions" -d 'Generate shell completion scripts'
complete -c warp -n "__fish_warp_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -s p -l page -d 'Page number' -r
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -s s -l size -d 'Results per page' -r
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -s t -l law-type -d 'Law type filter' -r
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -s d -l department -d 'Department filter' -r
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -a "search" -d 'Search for laws'
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -a "detail" -d 'Get law details'
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -a "history" -d 'Get law history'
complete -c warp -n "__fish_warp_using_subcommand law; and not __fish_seen_subcommand_from search detail history help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from search" -s p -l page -d 'Page number' -r
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from search" -s s -l size -d 'Results per page' -r
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from search" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from search" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from detail" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from detail" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from detail" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from history" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from history" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from history" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search for laws'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from help" -f -a "detail" -d 'Get law details'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from help" -f -a "history" -d 'Get law history'
complete -c warp -n "__fish_warp_using_subcommand law; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -s p -l page -d 'Page number' -r
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -s s -l size -d 'Results per page' -r
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -s r -l region -d 'Region filter' -r
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -s t -l law-type -d 'Law type filter' -r
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -a "search" -d 'Search for ordinances'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -a "detail" -d 'Get ordinance details'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and not __fish_seen_subcommand_from search detail help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from search" -s p -l page -d 'Page number' -r
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from search" -s s -l size -d 'Results per page' -r
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from search" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from search" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from detail" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from detail" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from detail" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search for ordinances'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from help" -f -a "detail" -d 'Get ordinance details'
complete -c warp -n "__fish_warp_using_subcommand ordinance; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -s p -l page -d 'Page number' -r
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -s s -l size -d 'Results per page' -r
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -s c -l court -d 'Court filter' -r
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -s t -l case-type -d 'Case type filter' -r
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -l date-from -d 'Date from (YYYYMMDD)' -r
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -l date-to -d 'Date to (YYYYMMDD)' -r
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -a "search" -d 'Search for precedents'
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -a "detail" -d 'Get precedent details'
complete -c warp -n "__fish_warp_using_subcommand precedent; and not __fish_seen_subcommand_from search detail help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from search" -s p -l page -d 'Page number' -r
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from search" -s s -l size -d 'Results per page' -r
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from search" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from search" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from detail" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from detail" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from detail" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search for precedents'
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from help" -f -a "detail" -d 'Get precedent details'
complete -c warp -n "__fish_warp_using_subcommand precedent; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand admrule" -s p -l page -d 'Page number' -r
complete -c warp -n "__fish_warp_using_subcommand admrule" -s s -l size -d 'Results per page' -r
complete -c warp -n "__fish_warp_using_subcommand admrule" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand admrule" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand admrule" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand interpretation" -s p -l page -d 'Page number' -r
complete -c warp -n "__fish_warp_using_subcommand interpretation" -s s -l size -d 'Results per page' -r
complete -c warp -n "__fish_warp_using_subcommand interpretation" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand interpretation" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand interpretation" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand search" -s p -l page -d 'Page number' -r
complete -c warp -n "__fish_warp_using_subcommand search" -s s -l size -d 'Results per page' -r
complete -c warp -n "__fish_warp_using_subcommand search" -s S -l source -d 'Source to search (nlic, elis, all)' -r
complete -c warp -n "__fish_warp_using_subcommand search" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand search" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand config; and not __fish_seen_subcommand_from set get path init help" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand config; and not __fish_seen_subcommand_from set get path init help" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand config; and not __fish_seen_subcommand_from set get path init help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand config; and not __fish_seen_subcommand_from set get path init help" -f -a "set" -d 'Set a configuration value'
complete -c warp -n "__fish_warp_using_subcommand config; and not __fish_seen_subcommand_from set get path init help" -f -a "get" -d 'Get a configuration value'
complete -c warp -n "__fish_warp_using_subcommand config; and not __fish_seen_subcommand_from set get path init help" -f -a "path" -d 'Show configuration file path'
complete -c warp -n "__fish_warp_using_subcommand config; and not __fish_seen_subcommand_from set get path init help" -f -a "init" -d 'Initialize configuration'
complete -c warp -n "__fish_warp_using_subcommand config; and not __fish_seen_subcommand_from set get path init help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from set" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from set" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from get" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from path" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from path" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from path" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from init" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from init" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from init" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "set" -d 'Set a configuration value'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get a configuration value'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "path" -d 'Show configuration file path'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "init" -d 'Initialize configuration'
complete -c warp -n "__fish_warp_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand version" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand version" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand version" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand completions" -s f -l format -d 'Output format' -r -f -a "table\t'Table format (default)'
json\t'JSON format'
markdown\t'Markdown format'
csv\t'CSV format'
html\t'HTML format'
html-simple\t'Simple HTML format'"
complete -c warp -n "__fish_warp_using_subcommand completions" -s v -l verbose -d 'Enable verbose output'
complete -c warp -n "__fish_warp_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "law" -d 'Search and view laws (국가법령)'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "ordinance" -d 'Search and view local ordinances (자치법규)'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "precedent" -d 'Search precedents (판례)'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "admrule" -d 'Search administrative rules (행정규칙)'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "interpretation" -d 'Search legal interpretations (법령해석례)'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "search" -d 'Unified search across all sources'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "config" -d 'Manage configuration'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "version" -d 'Show version information'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "completions" -d 'Generate shell completion scripts'
complete -c warp -n "__fish_warp_using_subcommand help; and not __fish_seen_subcommand_from law ordinance precedent admrule interpretation search config version completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from law" -f -a "search" -d 'Search for laws'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from law" -f -a "detail" -d 'Get law details'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from law" -f -a "history" -d 'Get law history'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from ordinance" -f -a "search" -d 'Search for ordinances'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from ordinance" -f -a "detail" -d 'Get ordinance details'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from precedent" -f -a "search" -d 'Search for precedents'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from precedent" -f -a "detail" -d 'Get precedent details'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "set" -d 'Set a configuration value'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "get" -d 'Get a configuration value'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "path" -d 'Show configuration file path'
complete -c warp -n "__fish_warp_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "init" -d 'Initialize configuration'
