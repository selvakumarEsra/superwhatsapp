self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.superwhatsapp;
  json = pkgs.formats.json { };
  # paths.rs: ProjectDirs::from("me", "paolino", "superwhatsapp").
  configDir =
    if pkgs.stdenv.hostPlatform.isDarwin then
      "${config.home.homeDirectory}/Library/Application Support/cc.superwhatsapp"
    else
      "${config.xdg.configHome}/superwhatsapp";
  declared = json.generate "superwhatsapp-settings.json" cfg.settings;
  jq = lib.getExe pkgs.jq;
  readSecret = key: path: ''
    if [[ -r ${lib.escapeShellArg path} ]]; then
      superwhatsappSecrets+=(--rawfile ${lib.escapeShellArg key} ${lib.escapeShellArg path})
    else
      warnEcho ${lib.escapeShellArg "SuperWhatsapp: cannot read ${path}, keeping the current ${key}"}
    fi
  '';
in
{
  options.programs.superwhatsapp = {
    enable = lib.mkEnableOption "SuperWhatsapp, a native WhatsApp client";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "superwhatsapp.packages.\${pkgs.stdenv.hostPlatform.system}.default";
      description = "The SuperWhatsapp package to install.";
    };

    settings = lib.mkOption {
      type = lib.types.attrsOf json.type;
      default = { };
      example = lib.literalExpression ''
        {
          check_for_updates = false;
          message_sound = "none";
          theme = "dark";
        }
      '';
      description = ''
        Keys merged into SuperWhatsapp's `settings.json` on every activation. Keys
        left out keep whatever the app last saved, so window geometry, recent
        emoji and the lock verifiers survive. A key declared here is reset on
        the next activation if it is changed in the app. Per-account preferences
        such as read receipts live in each account's own file and cannot be
        declared here. Restart SuperWhatsapp after
        activation: it reads the file at start, and its next save writes back
        the values it holds in memory.

        Values are copied to the Nix store, which every local user can read.
        Give secrets, such as proxy credentials or `giphy_key`, through
        {option}`programs.superwhatsapp.secretFiles` instead.
      '';
    };

    secretFiles = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      example = lib.literalExpression ''
        {
          proxy = config.sops.secrets.superwhatsapp-proxy.path;
        }
      '';
      description = ''
        Keys of `settings.json` whose string values are read at activation from
        these files, for example secrets decrypted by sops-nix or agenix, so
        they never enter the Nix store. Give each path as a string: a Nix path
        literal would be copied to the store. A trailing newline is dropped. A
        file that cannot be read leaves that key as it is, with a warning; a
        secret manager that decrypts after activation fills it on the next
        activation.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = builtins.all (key: !(cfg.settings ? ${key})) (builtins.attrNames cfg.secretFiles);
        message = "programs.superwhatsapp: a key cannot be in both settings and secretFiles.";
      }
    ];

    home.packages = [ cfg.package ];

    # settings.json cannot be a store symlink: the app replaces it whole on
    # every change and keeps its own state in it.
    home.activation.superwhatsappSettings = lib.mkIf (cfg.settings != { } || cfg.secretFiles != { }) (
      lib.hm.dag.entryAfter [ "writeBoundary" ] ''
        superwhatsappDir=${lib.escapeShellArg configDir}
        superwhatsappFile="$superwhatsappDir/settings.json"
        run mkdir -p -m 700 "$superwhatsappDir"
        if [[ ! -e "$superwhatsappFile" ]]; then
          # A file without a version is migrated as one from older releases,
          # which would replace an explicit beige or black wallpaper.
          superwhatsappBase='{"version":1}'
        elif superwhatsappBase=$(${jq} -ce objects "$superwhatsappFile" 2>/dev/null); then
          :
        else
          warnEcho "SuperWhatsapp: $superwhatsappFile is not a JSON object, leaving it unchanged"
          superwhatsappBase=
        fi
        if [[ -z "$superwhatsappBase" ]]; then
          :
        elif [[ -v DRY_RUN ]]; then
          echo "Would merge ${declared} into $superwhatsappFile"
        else
          superwhatsappSecrets=()
          ${lib.concatStrings (lib.mapAttrsToList readSecret cfg.secretFiles)}
          superwhatsappTmp=$(mktemp "$superwhatsappDir/.settings.json.XXXXXX")
          ${jq} --slurpfile __superwhatsappDeclared ${declared} "''${superwhatsappSecrets[@]}" \
            '. * $__superwhatsappDeclared[0] * ($ARGS.named | del(.__superwhatsappDeclared) | map_values(rtrimstr("\n")))' \
            <<< "$superwhatsappBase" > "$superwhatsappTmp"
          chmod 600 "$superwhatsappTmp"
          mv "$superwhatsappTmp" "$superwhatsappFile"
        fi
        unset superwhatsappDir superwhatsappFile superwhatsappBase superwhatsappSecrets superwhatsappTmp
      ''
    );
  };
}
