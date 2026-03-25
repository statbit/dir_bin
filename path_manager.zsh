#!/usr/bin/env zsh

# Path Manager Plugin for zsh
# Automatically manages PATH to include directory-specific bin directories

# Function to remove all existing dir_bin entries from PATH
_remove_dir_bins_from_path() {
    local new_path=""
    local IFS=":"

    # Split PATH and rebuild without dir_bin entries
    for path_entry in ${(s/:/)PATH}; do
        if [[ "$path_entry" != *"/bin/dir_bin/"* ]]; then
            if [[ -n "$new_path" ]]; then
                new_path="$new_path:$path_entry"
            else
                new_path="$path_entry"
            fi
        fi
    done

    export PATH="$new_path"
}

# Function to add current directory's bin to PATH
_add_current_dir_bin_to_path() {
    local current_dir="$(basename "$PWD")"
    local dir_bin_path="$HOME/bin/dir_bin/$current_dir"

    # Only add to PATH if the directory exists
    if [[ -d "$dir_bin_path" ]]; then
        export PATH="$dir_bin_path:$PATH"
    fi
}

# Function to remove all existing dir_bin aliases
_remove_dir_bin_aliases() {
    # Get all aliases that start with __dir_bin_
    local aliases_to_remove=(${(k)aliases[(I)__dir_bin_*]})

    # Unset each alias
    for alias_name in $aliases_to_remove; do
        unalias $alias_name 2>/dev/null || true
    done
}

# Function to add current directory's aliases
_add_current_dir_aliases() {
    local current_dir="$(basename "$PWD")"
    local aliases_file="$HOME/bin/dir_bin/$current_dir/__aliases"

    # Only process aliases if the file exists
    if [[ -f "$aliases_file" ]]; then
        # Read the aliases file and set up each alias
        while IFS= read -r line; do
            # Skip empty lines and comments
            if [[ -n "$line" && "$line" != \#* ]]; then
                # Parse alias in format: cmd="original cmd"
                if [[ "$line" =~ ^([^=]+)=(.+)$ ]]; then
                    local alias_name="${match[1]}"
                    local alias_value="${match[2]}"

                    # Remove quotes from the value if present
                    alias_value="${alias_value%\"}"
                    alias_value="${alias_value#\"}"
                    alias_value="${alias_value%\'}"
                    alias_value="${alias_value#\'}"

                    # Set the alias with a prefix to track it
                    alias "__dir_bin_$alias_name=$alias_value"
                fi
            fi
        done < "$aliases_file"
    fi
}

# Function to unset bindings from previous directory
_unset_previous_bindings() {
    local dir_name="$1"

    # Only proceed if directory name is provided and not empty
    if [[ -z "$dir_name" ]]; then
        return
    fi

    local bindings_file="$HOME/bin/dir_bin/$dir_name/__bindings.zsh"

    # Source the bindings file with 'unset' argument if it exists
    if [[ -f "$bindings_file" ]]; then
        source "$bindings_file" "unset"
    fi
}

# Function to set bindings for current directory
_set_current_bindings() {
    local dir_name="$1"

    # Only proceed if directory name is provided and not empty
    if [[ -z "$dir_name" ]]; then
        return
    fi

    local bindings_file="$HOME/bin/dir_bin/$dir_name/__bindings.zsh"

    # Source the bindings file with 'set' argument if it exists
    if [[ -f "$bindings_file" ]]; then
        source "$bindings_file" "set"
    fi
}

# Function called whenever directory changes
_path_manager_chpwd() {
    # Always clean up existing dir_bin aliases first
    _remove_dir_bin_aliases

    local current_dir="$(basename "$PWD")"
    local previous_dir="$(basename "$OLDPWD")"

    # Only manage PATH and aliases if we're under $HOME but not in the dir_bin directory itself
    if [[ "$PWD" == "$HOME"* && "$PWD" != "$HOME/bin/dir_bin"* ]]; then
        # Remove any existing dir_bin entries
        _remove_dir_bins_from_path

        # Add current directory's bin if it exists
        _add_current_dir_bin_to_path

        # Add current directory's aliases if they exist
        _add_current_dir_aliases

        _unset_previous_bindings "$previous_dir"
        _set_current_bindings "$current_dir"
    else
        # If we're outside $HOME or in dir_bin directory, just clean up any existing dir_bin entries
        _remove_dir_bins_from_path
    fi
}

# Hook into directory changes
autoload -U add-zsh-hook
add-zsh-hook chpwd _path_manager_chpwd

# Initialize for current directory when plugin loads
_path_manager_chpwd

# User-facing command to manually reinitialize the path manager
# Useful if initialization fails on shell startup
refresh-dir-bin() {
    _path_manager_chpwd
}
