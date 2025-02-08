export default function AdminMessage() {
  return (
    <div className="flex min-h-screen items-center justify-center">
      <div className="max-w-md p-8 text-center space-y-4">
        <h1 className="text-2xl font-bold text-gray-900">
          Acceso no permitido
        </h1>
        <p className="text-gray-600">
          Esta aplicación está diseñada exclusivamente para vendedores. Los
          administradores deben utilizar la aplicación web.
        </p>
        <a
          href="/"
          className="inline-block text-sm text-emerald-600 hover:text-emerald-700"
        >
          Volver al inicio
        </a>
      </div>
    </div>
  );
}
