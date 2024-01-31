{ pkgs, config, lib, ... }:
{
  security.acme = {
    acceptTerms = true;
    defaults.email = "marijan.petricevic94@gmail.com";
  };
}
