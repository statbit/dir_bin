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

# Function called whenever directory changes
_path_manager_chpwd() {
    # Only manage PATH if we're under $HOME but not in the dir_bin directory itself
    if [[ "$PWD" == "$HOME"* && "$PWD" != "$HOME/bin/dir_bin"* ]]; then
        # Remove any existing dir_bin entries
        _remove_dir_bins_from_path
        
        # Add current directory's bin if it exists
        _add_current_dir_bin_to_path
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
