@echo off
title Servidor Verificador de Precios Gsigma
echo ========================================================
echo   Servidor Verificador de Precios Local para Gsigma
echo ========================================================
echo.
echo Iniciando servicio en puerto 8080...
echo Para acceder desde la Tablet, abra el navegador y entre a:
echo http://<IP_DE_ESTA_PC>:8080
echo.
cd /d "%~dp0"
target\release\verifgsigma.exe
pause
