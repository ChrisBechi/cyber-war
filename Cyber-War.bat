@echo off
setlocal EnableExtensions DisableDelayedExpansion
title Cyber War - Testar e gerar release

pushd "%~dp0"
if errorlevel 1 (
    echo Nao foi possivel acessar a pasta do projeto.
    pause
    exit /b 1
)

if not exist "package.json" goto missing_project
if not exist "scripts\dev.ps1" goto missing_project

echo.
echo =============================================
echo                  CYBER WAR
echo =============================================
echo.
echo [1] Instalar dependencias e testar o jogo
echo [2] Instalar dependencias, gerar release e abrir
echo [3] Abrir o release ja compilado
echo [0] Sair
echo.
echo A opcao 3 nao recompila as alteracoes do codigo.
echo.
choice /C 1230 /N /M "Escolha uma opcao: "
if errorlevel 4 goto finish
if errorlevel 3 goto launch
if errorlevel 2 goto build
if errorlevel 1 goto dev
goto finish

:dev
set "CYBER_TASK=dev"
goto run

:build
set "CYBER_TASK=build"
goto run

:run
call :prepare_tools
if errorlevel 1 goto failed

echo.
echo Instalando as dependencias. Aguarde a conclusao.
call pnpm install --frozen-lockfile
if errorlevel 1 goto failed

echo.
if "%CYBER_TASK%"=="dev" echo Abrindo o jogo para testes. Mantenha esta janela aberta.
if "%CYBER_TASK%"=="build" echo Compilando o release e os instaladores. Aguarde a conclusao.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\dev.ps1" -Task "%CYBER_TASK%"
if errorlevel 1 goto failed

if "%CYBER_TASK%"=="build" goto launch
goto finish

:launch
if not exist "%~dp0src-tauri\target\release\game-hacker.exe" (
    echo.
    echo Nenhum release foi encontrado. Execute este arquivo e escolha a opcao 2.
    pause
    popd
    exit /b 1
)
echo.
echo Abrindo "%~dp0src-tauri\target\release\game-hacker.exe"
start "" /D "%~dp0src-tauri\target\release" "%~dp0src-tauri\target\release\game-hacker.exe"
if errorlevel 1 goto failed
goto finish

:prepare_tools
where node >nul 2>nul
if not errorlevel 1 goto prepare_pnpm
if exist "%ProgramFiles%\nodejs\node.exe" set "PATH=%ProgramFiles%\nodejs;%PATH%"
where node >nul 2>nul
if not errorlevel 1 goto prepare_pnpm
if exist "%USERPROFILE%\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe" set "PATH=%USERPROFILE%\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin;%PATH%"
where node >nul 2>nul
if not errorlevel 1 goto prepare_pnpm
echo Node.js nao foi encontrado. Instale o Node.js 24 antes de continuar.
exit /b 1

:prepare_pnpm
where pnpm >nul 2>nul
if not errorlevel 1 exit /b 0
if exist "%USERPROFILE%\.cache\codex-runtimes\codex-primary-runtime\dependencies\bin\fallback\pnpm.cmd" set "PATH=%USERPROFILE%\.cache\codex-runtimes\codex-primary-runtime\dependencies\bin\fallback;%PATH%"
where pnpm >nul 2>nul
if not errorlevel 1 exit /b 0
echo pnpm nao foi encontrado. Instale o pnpm 11.19.0 antes de continuar.
exit /b 1

:missing_project
echo.
echo Mantenha este arquivo na pasta principal do Cyber War, ao lado de package.json.
pause
popd
exit /b 1

:failed
set "CYBER_EXIT_CODE=%ERRORLEVEL%"
echo.
echo A operacao falhou. Confira a mensagem de erro acima.
echo Nenhuma etapa seguinte sera executada automaticamente.
pause
popd
exit /b %CYBER_EXIT_CODE%

:finish
popd
exit /b 0
