# language: es
Característica: Configuración del espacio
  Para administrar un espacio desde /settings
  Como administrador de plataforma o de espacio
  Quiero consultar y cambiar la membresía y la aprobación de usuarios

  Antecedentes:
    Dado un espacio "barrio-test" con datos mínimos
    Y un administrador de plataforma autenticado
    Y un usuario regular autenticado con perfil completo

  Escenario: membership/me responde para el administrador
    Cuando el administrador consulta su membresía
    Entonces la respuesta es exitosa
    Y puede administrar el espacio

  Escenario: membership/me responde para un usuario regular
    Cuando el usuario regular consulta su membresía
    Entonces la respuesta es exitosa
    Y no puede administrar el espacio

  Escenario: Admin activa y desactiva la aprobación de usuarios
    Cuando el administrador activa la aprobación de usuarios
    Entonces el espacio requiere aprobación de usuarios
    Cuando el administrador desactiva la aprobación de usuarios
    Entonces el espacio no requiere aprobación de usuarios

  Escenario: Admin puede listar miembros del espacio
    Dado que el espacio requiere aprobación de usuarios
    Cuando el usuario regular solicita autorización
    Y el administrador lista los miembros pendientes
    Entonces la respuesta es exitosa
    Y la lista de miembros incluye al usuario regular

  Escenario: Usuario regular no puede listar miembros
    Cuando el usuario regular lista los miembros
    Entonces la respuesta es prohibida

  Escenario: Admin de espacio puede administrar el espacio
    Dado un admin de espacio autenticado
    Cuando el admin de espacio consulta su membresía
    Entonces la respuesta es exitosa
    Y puede administrar el espacio
    Cuando el admin de espacio lista los miembros
    Entonces la respuesta es exitosa
