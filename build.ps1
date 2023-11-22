Remove-Item -Recurse -Force publish
cd src/Qynit.PulseGen.Server
npm install
npm run build
cd ../..
dotnet publish ./src/Qynit.PulseGen.Server/ -c Release -o publish
